#!/usr/bin/env bash
# Build a bootable ISO: Limine BIOS + kernel ELF.
# Limine binaries and CLI source are vendored in vendor/limine/.
# No network access is required.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${OUT:-$ROOT/out}"
VENDOR_LIMINE="$ROOT/vendor/limine"
CC_BIN="${CC:-cc}"
ISO_TOOL="${XORRISO:-xorriso}"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
ISO_ROOT="$(mktemp -d "$OUT/iso_root.XXXXXX")"
trap 'rm -rf "$ISO_ROOT"' EXIT

# Callers can override which kernel ELF and output ISO name to use.
KERNEL_ELF="${KERNEL_ELF:-kernel.elf}"
ISO_NAME="${ISO_NAME:-os.iso}"
SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-1}"
if [ -n "$SOURCE_DATE_EPOCH" ]; then
    export SOURCE_DATE_EPOCH
fi

# --- Build Limine CLI from vendored source (if needed) ------------------------
LIMINE_CLI="$OUT/limine-cli"
if [ -f "$OUT/limine-cli.exe" ]; then
    LIMINE_CLI="$OUT/limine-cli.exe"
fi
if [ ! -f "$LIMINE_CLI" ]; then
    if ! command -v "$CC_BIN" >/dev/null 2>&1; then
        if command -v gcc >/dev/null 2>&1; then
            CC_BIN="gcc"
        elif command -v gcc.exe >/dev/null 2>&1; then
            CC_BIN="gcc.exe"
        elif command -v clang >/dev/null 2>&1; then
            CC_BIN="clang"
        elif command -v clang.exe >/dev/null 2>&1; then
            CC_BIN="clang.exe"
        elif command -v cc.exe >/dev/null 2>&1; then
            CC_BIN="cc.exe"
        else
            echo "ERROR: no C compiler found (tried '$CC_BIN', gcc/gcc.exe, clang/clang.exe)." >&2
            exit 1
        fi
    fi
    echo "==> Building Limine CLI from vendored source..."
    mkdir -p "$OUT"
    (
        cd "$ROOT"
        "$CC_BIN" -O2 -pipe -Wall -Wextra -std=c99 -o "${LIMINE_CLI#"$ROOT"/}" "vendor/limine/limine.c"
    )
    if [ -f "$OUT/limine-cli.exe" ]; then
        LIMINE_CLI="$OUT/limine-cli.exe"
    fi
fi

# --- Assemble ISO tree --------------------------------------------------------
mkdir -p "$ISO_ROOT/boot/limine"

cp "$OUT/$KERNEL_ELF"                       "$ISO_ROOT/boot/kernel.elf"
cp "$ROOT/boot/limine.conf"                "$ISO_ROOT/boot/limine/limine.conf"
cp "$VENDOR_LIMINE/limine-bios.sys"        "$ISO_ROOT/boot/limine/"
cp "$VENDOR_LIMINE/limine-bios-cd.bin"     "$ISO_ROOT/boot/limine/"
XORRISO_DATE_ARGS=()
if [ -n "$SOURCE_DATE_EPOCH" ]; then
    # Normalize all input mtimes for reproducible ISO trees.
    find "$ISO_ROOT" -exec touch -h -d "@$SOURCE_DATE_EPOCH" {} +
    # Keep mkisofs-compatible args empty: some xorriso builds reject
    # --modification-date/-volume_date in -as mkisofs mode.
fi

# --- Create ISO ---------------------------------------------------------------
USED_PYCDLIB=0
if ! command -v "$ISO_TOOL" >/dev/null 2>&1; then
    if command -v mkisofs >/dev/null 2>&1; then
        ISO_TOOL="mkisofs"
    elif command -v genisoimage >/dev/null 2>&1; then
        ISO_TOOL="genisoimage"
    elif command -v python3 >/dev/null 2>&1 || command -v python >/dev/null 2>&1; then
        PYTHON_BIN="python3"
        if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
            PYTHON_BIN="python"
        fi
        echo "==> No xorriso/mkisofs found; using pycdlib fallback ISO builder..."
        "$PYTHON_BIN" "$ROOT/tools/mkiso_pycdlib.py" \
            --iso-root "$ISO_ROOT" \
            --out-iso "$OUT/$ISO_NAME"
        USED_PYCDLIB=1
    else
        echo "ERROR: no ISO builder found (tried '$ISO_TOOL', mkisofs, genisoimage, pycdlib)." >&2
        exit 1
    fi
fi

if [ "$USED_PYCDLIB" -eq 0 ]; then
    if [ "$ISO_TOOL" = "xorriso" ]; then
        "$ISO_TOOL" -as mkisofs \
            -R -r -J \
            -V "RUGO_OS" -volset "RUGO_OS" -A "RUGO_OS" -p "RUGO" -P "RUGO" \
            "${XORRISO_DATE_ARGS[@]}" \
            -b boot/limine/limine-bios-cd.bin \
            -no-emul-boot -boot-load-size 4 -boot-info-table \
            -o "$OUT/$ISO_NAME" \
            "$ISO_ROOT"
    else
        # xorriso date flags are not supported by mkisofs/genisoimage.
        "$ISO_TOOL" \
            -R -r -J \
            -V "RUGO_OS" -volset "RUGO_OS" -A "RUGO_OS" -p "RUGO" -P "RUGO" \
            -b boot/limine/limine-bios-cd.bin \
            -no-emul-boot -boot-load-size 4 -boot-info-table \
            -o "$OUT/$ISO_NAME" \
            "$ISO_ROOT"
    fi
fi

# --- Install Limine BIOS boot stages -----------------------------------------
ISO_FOR_LIMINE="$OUT/$ISO_NAME"
if [[ "$LIMINE_CLI" == *.exe ]] && command -v wslpath >/dev/null 2>&1; then
    ISO_FOR_LIMINE="$(wslpath -w "$ISO_FOR_LIMINE")"
fi
"$LIMINE_CLI" bios-install "$ISO_FOR_LIMINE"

echo "==> Image ready: $OUT/$ISO_NAME"
