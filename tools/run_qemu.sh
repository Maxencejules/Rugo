#!/usr/bin/env bash
# Canonical QEMU runner for Rugo.
# Usage: ./tools/run_qemu.sh [--iso PATH] [PATH]
#
# This is the single entry point for launching the OS in QEMU.
# Used by: make run, make run-kernel, make demo-go, make test-qemu, CI workflows.

set -euo pipefail

ISO="out/os.iso"

if [ "${1:-}" = "--iso" ]; then
    if [ -z "${2:-}" ]; then
        echo "error: --iso requires a path"
        exit 1
    fi
    ISO="$2"
elif [ -n "${1:-}" ]; then
    ISO="$1"
fi

if [ ! -f "$ISO" ]; then
    echo "error: image not found at $ISO"
    echo "Run 'make image', 'make run-kernel', or 'make demo-go' first."
    exit 1
fi

resolve_qemu_bin() {
    local candidate=""

    if [ -n "${QEMU_BIN:-}" ] && [ -f "${QEMU_BIN}" ]; then
        printf '%s\n' "${QEMU_BIN}"
        return 0
    fi

    if candidate="$(command -v qemu-system-x86_64 2>/dev/null)"; then
        printf '%s\n' "${candidate}"
        return 0
    fi

    if candidate="$(command -v qemu-system-x86_64.exe 2>/dev/null)"; then
        printf '%s\n' "${candidate}"
        return 0
    fi

    for candidate in \
        "/mnt/c/Program Files/qemu/qemu-system-x86_64.exe" \
        "/mnt/c/Program Files (x86)/qemu/qemu-system-x86_64.exe" \
        "/c/Program Files/qemu/qemu-system-x86_64.exe" \
        "/c/Program Files (x86)/qemu/qemu-system-x86_64.exe"
    do
        if [ -f "${candidate}" ]; then
            printf '%s\n' "${candidate}"
            return 0
        fi
    done

    return 1
}

QEMU_BIN="$(resolve_qemu_bin || true)"
QEMU_DEBUG_FLAGS=()
QEMU_SUCCESS_EXIT="${QEMU_SUCCESS_EXIT:-99}"

if [ -z "${QEMU_BIN}" ]; then
    echo "error: qemu-system-x86_64 not found"
    echo "Install QEMU or set QEMU_BIN to the binary path."
    exit 1
fi

if [ -n "${QEMU_DEBUG:-}" ]; then
    QEMU_DEBUG_FLAGS=(-d "${QEMU_DEBUG}")
fi

# QEMU invocation from MILESTONES.md section 3.
# Adjust OVMF paths or remove pflash lines for BIOS-mode Limine boot.
set +e
"${QEMU_BIN}" \
    -machine q35 \
    -cpu qemu64 \
    -m 1024 \
    -serial stdio \
    -display none \
    -no-reboot \
    "${QEMU_DEBUG_FLAGS[@]}" \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -cdrom "$ISO"
status=$?
set -e

if [ "$status" -eq 0 ] || [ "$status" -eq "$QEMU_SUCCESS_EXIT" ]; then
    exit 0
fi

exit "$status"
