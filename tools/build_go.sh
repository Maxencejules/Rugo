#!/usr/bin/env bash
# Build TinyGo user-space binary for Rugo G1.
# Produces out/gousr.bin (flat binary for kernel embedding).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/out"
SVC="$ROOT/services/go"
mkdir -p "$OUT"

# In WSL, Windows-installed Go/TinyGo tools may only exist as .exe paths.
WINUSER="${USERNAME:-}"
if [ -z "$WINUSER" ] && [[ "$ROOT" =~ ^/mnt/c/Users/([^/]+)/ ]]; then
    WINUSER="${BASH_REMATCH[1]}"
fi
if [ -d "/mnt/c/Progra~1/Go/bin" ]; then
    PATH="/mnt/c/Progra~1/Go/bin:$PATH"
elif [ -d "/mnt/c/Program Files/Go/bin" ]; then
    PATH="/mnt/c/Program Files/Go/bin:$PATH"
fi
if [ -n "$WINUSER" ]; then
    TINYGO_WIN_BIN="/mnt/c/Users/$WINUSER/AppData/Local/Microsoft/WinGet/Packages/tinygo-org.tinygo_Microsoft.Winget.Source_8wekyb3d8bbwe/tinygo/bin"
    if [ -d "$TINYGO_WIN_BIN" ]; then
        PATH="$TINYGO_WIN_BIN:$PATH"
    fi
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

# --- Check prerequisites ---
TINYGO_BIN="$(find_tool tinygo tinygo.exe "${TINYGO_WIN_BIN:-}/tinygo.exe")" || {
    echo "ERROR: tinygo not found in PATH"
    exit 1
}
NASM_BIN="$(find_tool nasm nasm.exe /mnt/c/mingw64/mingw64/bin/nasm.exe)" || {
    echo "ERROR: nasm not found in PATH"
    exit 1
}
OBJCOPY_BIN="$(find_tool objcopy objcopy.exe /mnt/c/mingw64/mingw64/bin/objcopy.exe)" || {
    echo "ERROR: objcopy not found in PATH"
    exit 1
}

NASM_OUT="$OUT/go_start.o"
NASM_SRC="$SVC/start.asm"
TINYGO_TARGET="$OUT/rugo-target.json"
TINYGO_OUT="$OUT/gousr.elf"
OBJCOPY_IN="$OUT/gousr.elf"
OBJCOPY_OUT="$OUT/gousr.bin"

if [[ "$NASM_BIN" == *.exe ]]; then
    NASM_OUT="$(to_host_path "$NASM_OUT")"
    NASM_SRC="$(to_host_path "$NASM_SRC")"
fi
if [[ "$TINYGO_BIN" == *.exe ]]; then
    TINYGO_TARGET="$(to_host_path "$TINYGO_TARGET")"
    TINYGO_OUT="$(to_host_path "$TINYGO_OUT")"
fi
if [[ "$OBJCOPY_BIN" == *.exe ]]; then
    OBJCOPY_IN="$(to_host_path "$OBJCOPY_IN")"
    OBJCOPY_OUT="$(to_host_path "$OBJCOPY_OUT")"
fi

# --- Assemble startup + syscall stubs ---
echo "==> Assembling Go startup stubs..."
"$NASM_BIN" -f elf64 -o "$NASM_OUT" "$NASM_SRC"

# --- Generate target JSON with absolute paths ---
# TinyGo ldflags need absolute paths since LLD CWD is unpredictable.
# Use forward slashes which work on both Unix and Windows in LLD.
START_OBJ="$OUT/go_start.o"
LINKER_LD="$SVC/linker.ld"
if [[ "$TINYGO_BIN" == *.exe ]]; then
    START_OBJ="$(to_host_path "$START_OBJ")"
    LINKER_LD="$(to_host_path "$LINKER_LD")"
fi

cat > "$OUT/rugo-target.json" <<ENDJSON
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
"$TINYGO_BIN" build -target="$TINYGO_TARGET" -no-debug -o "$TINYGO_OUT" .

# --- Convert to flat binary ---
echo "==> Converting to flat binary..."
"$OBJCOPY_BIN" -O binary "$OBJCOPY_IN" "$OBJCOPY_OUT"
BIN_SIZE=$(stat -c%s "$OUT/gousr.bin" 2>/dev/null || stat -f%z "$OUT/gousr.bin" 2>/dev/null || wc -c < "$OUT/gousr.bin")
if [ "$BIN_SIZE" -gt 4096 ]; then
    echo "ERROR: Go user binary exceeds the current 4096-byte userspace image limit ($BIN_SIZE bytes)"
    exit 1
fi
echo "==> Go user binary: $OUT/gousr.bin ($BIN_SIZE bytes)"
