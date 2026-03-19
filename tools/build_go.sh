#!/usr/bin/env bash
# Build TinyGo user-space binary for Rugo G1.
# Produces out/gousr.bin (flat binary for kernel embedding).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/out"
SVC="$ROOT/services/go"
OUT_ELF="$OUT/gousr.elf"
OUT_BIN="$OUT/gousr.bin"
TARGET_JSON="$OUT/rugo-target.json"
BUILD_TAGS=()
MAX_BIN_BYTES=24576

while [ "$#" -gt 0 ]; do
    case "$1" in
        --service-dir)
            SVC="$ROOT/${2:-}"
            shift 2
            ;;
        --out-elf)
            OUT_ELF="$ROOT/${2:-}"
            shift 2
            ;;
        --out-bin)
            OUT_BIN="$ROOT/${2:-}"
            shift 2
            ;;
        --target-json)
            TARGET_JSON="$ROOT/${2:-}"
            shift 2
            ;;
        --build-tags)
            BUILD_TAGS+=("${2:-}")
            shift 2
            ;;
        --max-bin-bytes)
            MAX_BIN_BYTES="${2:-}"
            shift 2
            ;;
        *)
            echo "error: unknown argument: $1"
            exit 1
            ;;
    esac
done

mkdir -p "$OUT"

# In WSL, Windows-installed Go/TinyGo tools may only exist as .exe paths.
WINUSER="${USERNAME:-}"
if [ -z "$WINUSER" ] && [[ "$ROOT" =~ ^/mnt/c/Users/([^/]+)/ ]]; then
    WINUSER="${BASH_REMATCH[1]}"
fi
if [ -z "$WINUSER" ] && [[ "$ROOT" =~ ^/c/Users/([^/]+)/ ]]; then
    WINUSER="${BASH_REMATCH[1]}"
fi

prepend_path_if_dir() {
    if [ -d "$1" ]; then
        PATH="$1:$PATH"
    fi
}

prepend_compatible_go_sdk() {
    local root
    local candidate
    for root in "$@"; do
        [ -d "$root" ] || continue
        for candidate in "$root"/go1.25.* "$root"/go1.24.* "$root"/go1.23.* "$root"/go1.22.*; do
            if [ -d "$candidate/bin" ]; then
                PATH="$candidate/bin:$PATH"
                export GOTOOLCHAIN=local
                return 0
            fi
        done
    done
    return 1
}

prepend_path_if_dir "/mnt/c/Progra~1/Go/bin"
prepend_path_if_dir "/mnt/c/Program Files/Go/bin"
prepend_path_if_dir "/c/Progra~1/Go/bin"
prepend_path_if_dir "/c/Program Files/Go/bin"

MINGW_WIN_BIN="/mnt/c/mingw64/mingw64/bin"
MINGW_WIN_BIN_ALT="/c/mingw64/mingw64/bin"

if [ -d "$MINGW_WIN_BIN" ]; then
    PATH="$MINGW_WIN_BIN:$PATH"
elif [ -d "$MINGW_WIN_BIN_ALT" ]; then
    PATH="$MINGW_WIN_BIN_ALT:$PATH"
fi
if [ -n "$WINUSER" ]; then
    TINYGO_WIN_BIN="/mnt/c/Users/$WINUSER/AppData/Local/Microsoft/WinGet/Packages/tinygo-org.tinygo_Microsoft.Winget.Source_8wekyb3d8bbwe/tinygo/bin"
    TINYGO_WIN_BIN_ALT="/c/Users/$WINUSER/AppData/Local/Microsoft/WinGet/Packages/tinygo-org.tinygo_Microsoft.Winget.Source_8wekyb3d8bbwe/tinygo/bin"
    prepend_path_if_dir "$TINYGO_WIN_BIN"
    prepend_path_if_dir "$TINYGO_WIN_BIN_ALT"
    prepend_compatible_go_sdk "/mnt/c/Users/$WINUSER/sdk" "/c/Users/$WINUSER/sdk" || true
fi
export PATH

find_tool() {
    for cand in "$@"; do
        if command -v "$cand" >/dev/null 2>&1; then
            command -v "$cand"
            return 0
        fi
        if [ -x "$cand" ]; then
            echo "$cand"
            return 0
        fi
    done
    return 1
}

to_host_path() {
    if command -v cygpath >/dev/null 2>&1; then
        cygpath -m "$1"
    elif command -v wslpath >/dev/null 2>&1; then
        wslpath -m "$1"
    else
        echo "$1"
    fi
}

tool_uses_host_paths() {
    case "$1" in
        *.exe|/[a-zA-Z]/*)
            return 0
            ;;
    esac
    return 1
}

# --- Check prerequisites ---
TINYGO_BIN="$(find_tool tinygo tinygo.exe "${TINYGO_WIN_BIN:-}/tinygo.exe")" || {
    echo "ERROR: tinygo not found in PATH"
    exit 1
}
NASM_BIN="$(find_tool nasm nasm.exe "$MINGW_WIN_BIN/nasm.exe" "$MINGW_WIN_BIN_ALT/nasm.exe")" || {
    echo "ERROR: nasm not found in PATH"
    exit 1
}
OBJCOPY_BIN="$(find_tool objcopy objcopy.exe "$MINGW_WIN_BIN/objcopy.exe" "$MINGW_WIN_BIN_ALT/objcopy.exe")" || {
    echo "ERROR: objcopy not found in PATH"
    exit 1
}

SERVICE_NAME="$(basename "$SVC")"
NASM_OUT="$OUT/${SERVICE_NAME}_start.o"
NASM_SRC="$SVC/start.asm"
TINYGO_TARGET="$TARGET_JSON"
TINYGO_OUT="$OUT_ELF"
OBJCOPY_IN="$OUT_ELF"
OBJCOPY_OUT="$OUT_BIN"

if tool_uses_host_paths "$NASM_BIN"; then
    NASM_OUT="$(to_host_path "$NASM_OUT")"
    NASM_SRC="$(to_host_path "$NASM_SRC")"
fi
if tool_uses_host_paths "$TINYGO_BIN"; then
    TINYGO_TARGET="$(to_host_path "$TINYGO_TARGET")"
    TINYGO_OUT="$(to_host_path "$TINYGO_OUT")"
fi
if tool_uses_host_paths "$OBJCOPY_BIN"; then
    OBJCOPY_IN="$(to_host_path "$OBJCOPY_IN")"
    OBJCOPY_OUT="$(to_host_path "$OBJCOPY_OUT")"
fi

# --- Assemble startup + syscall stubs ---
echo "==> Assembling Go startup stubs..."
"$NASM_BIN" -f elf64 -o "$NASM_OUT" "$NASM_SRC"

# --- Generate target JSON with absolute paths ---
# TinyGo ldflags need absolute paths since LLD CWD is unpredictable.
# Use forward slashes which work on both Unix and Windows in LLD.
START_OBJ="$NASM_OUT"
LINKER_LD="$SVC/linker.ld"
if tool_uses_host_paths "$TINYGO_BIN"; then
    START_OBJ="$(to_host_path "$START_OBJ")"
    LINKER_LD="$(to_host_path "$LINKER_LD")"
fi

mkdir -p "$(dirname "$OUT_ELF")" "$(dirname "$OUT_BIN")" "$(dirname "$TARGET_JSON")"

cat > "$TARGET_JSON" <<ENDJSON
{
	"llvm-target": "x86_64-unknown-none-elf",
	"cpu": "x86-64",
	"features": "+soft-float,-mmx,-sse,-sse2,-avx",
	"build-tags": ["rugo", "linux", "amd64"],
	"goos": "linux",
	"goarch": "amd64",
	"gc": "none",
	"scheduler": "none",
	"linker": "ld.lld",
	"rtlib": "compiler-rt",
	"libc": "picolibc",
	"default-stack-size": 4096,
	"cflags": [
		"-mno-red-zone",
		"-mno-sse",
		"-fno-stack-protector",
		"-fno-exceptions",
		"-fno-unwind-tables",
		"-ffunction-sections",
		"-fdata-sections"
	],
	"ldflags": [
		"--entry=_start",
		"-nostdlib",
		"-T${LINKER_LD}",
		"${START_OBJ}"
	],
	"extra-files": []
}
ENDJSON

# --- Build with TinyGo ---
echo "==> Building Go user binary with TinyGo..."
cd "$SVC"
TINYGO_BUILD_ARGS=()
for tag in "${BUILD_TAGS[@]}"; do
    [ -n "$tag" ] || continue
    TINYGO_BUILD_ARGS+=("-tags=$tag")
done
"$TINYGO_BIN" build -target="$TINYGO_TARGET" -no-debug "${TINYGO_BUILD_ARGS[@]}" -o "$TINYGO_OUT" .

# --- Convert to flat binary ---
echo "==> Converting to flat binary..."
"$OBJCOPY_BIN" -O binary "$OBJCOPY_IN" "$OBJCOPY_OUT"
BIN_SIZE=$(stat -c%s "$OUT_BIN" 2>/dev/null || stat -f%z "$OUT_BIN" 2>/dev/null || wc -c < "$OUT_BIN")
if [ "$BIN_SIZE" -gt "$MAX_BIN_BYTES" ]; then
    echo "ERROR: Go user binary exceeds the current ${MAX_BIN_BYTES}-byte userspace image limit ($BIN_SIZE bytes)"
    exit 1
fi
echo "==> Go user binary: $OUT_BIN ($BIN_SIZE bytes)"
