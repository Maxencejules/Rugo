#!/usr/bin/env bash
# Canonical QEMU runner for Rugo.
# Usage: ./tools/run_qemu.sh [--iso PATH] [PATH]
#
# This is the single entry point for launching the OS in QEMU.
# Used by: make run, make run-kernel, make demo-go, make test-qemu, CI workflows.

set -euo pipefail

ISO="out/os.iso"
MACHINE="${QEMU_MACHINE:-q35}"
CPU="${QEMU_CPU:-qemu64}"
MEM_MB="${QEMU_MEM_MB:-1024}"
DISK_PATH=""
BLOCK_DEVICE="virtio-blk-pci,drive=disk0,disable-modern=on"
WITH_NET=0
NET_DEVICE="virtio-net-pci,netdev=n0,disable-modern=on"
STDIN_FILE=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --iso)
            if [ -z "${2:-}" ]; then
                echo "error: --iso requires a path"
                exit 1
            fi
            ISO="$2"
            shift 2
            ;;
        --disk)
            if [ -z "${2:-}" ]; then
                echo "error: --disk requires a path"
                exit 1
            fi
            DISK_PATH="$2"
            shift 2
            ;;
        --block-device)
            if [ -z "${2:-}" ]; then
                echo "error: --block-device requires a value"
                exit 1
            fi
            BLOCK_DEVICE="$2"
            shift 2
            ;;
        --with-net)
            WITH_NET=1
            shift
            ;;
        --net-device)
            if [ -z "${2:-}" ]; then
                echo "error: --net-device requires a value"
                exit 1
            fi
            NET_DEVICE="$2"
            shift 2
            ;;
        --machine)
            if [ -z "${2:-}" ]; then
                echo "error: --machine requires a value"
                exit 1
            fi
            MACHINE="$2"
            shift 2
            ;;
        --cpu)
            if [ -z "${2:-}" ]; then
                echo "error: --cpu requires a value"
                exit 1
            fi
            CPU="$2"
            shift 2
            ;;
        --mem)
            if [ -z "${2:-}" ]; then
                echo "error: --mem requires a value"
                exit 1
            fi
            MEM_MB="$2"
            shift 2
            ;;
        --stdin-file)
            if [ -z "${2:-}" ]; then
                echo "error: --stdin-file requires a path"
                exit 1
            fi
            STDIN_FILE="$2"
            shift 2
            ;;
        --help|-h)
            cat <<'EOF'
usage: ./tools/run_qemu.sh [--iso PATH] [--disk PATH] [--block-device VALUE] [--with-net]
                           [--net-device VALUE] [--machine VALUE] [--cpu VALUE]
                           [--mem MB] [--stdin-file PATH]
EOF
            exit 0
            ;;
        *)
            ISO="$1"
            shift
            ;;
    esac
done

if [ ! -f "$ISO" ]; then
    echo "error: image not found at $ISO"
    echo "Run 'make image', 'make run-kernel', or 'make demo-go' first."
    exit 1
fi

if [ -n "$STDIN_FILE" ] && [ ! -f "$STDIN_FILE" ]; then
    echo "error: stdin file not found at $STDIN_FILE"
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

resolve_python_bin() {
    local candidate=""

    if [ -n "${PYTHON_BIN:-}" ] && [ -f "${PYTHON_BIN}" ]; then
        printf '%s\n' "${PYTHON_BIN}"
        return 0
    fi

    if candidate="$(command -v python3 2>/dev/null)"; then
        printf '%s\n' "${candidate}"
        return 0
    fi

    if candidate="$(command -v python 2>/dev/null)"; then
        printf '%s\n' "${candidate}"
        return 0
    fi

    if candidate="$(command -v py 2>/dev/null)"; then
        printf '%s\n' "${candidate}"
        return 0
    fi

    for candidate in \
        "/c/Users/USER/AppData/Local/Programs/Python/Python312/python.exe" \
        "/c/Users/USER/AppData/Local/Microsoft/WindowsApps/python3.exe" \
        "/c/Users/USER/AppData/Local/Microsoft/WindowsApps/python.exe"
    do
        if [ -f "${candidate}" ]; then
            printf '%s\n' "${candidate}"
            return 0
        fi
    done

    return 1
}

QEMU_BIN="$(resolve_qemu_bin || true)"
PYTHON_RUNNER="$(resolve_python_bin || true)"
QEMU_DEBUG_FLAGS=()
QEMU_SUCCESS_EXIT="${QEMU_SUCCESS_EXIT:-99}"

if [ -z "${QEMU_BIN}" ]; then
    echo "error: qemu-system-x86_64 not found"
    echo "Install QEMU or set QEMU_BIN to the binary path."
    exit 1
fi

if [ -n "$STDIN_FILE" ] && [ -z "${PYTHON_RUNNER}" ]; then
    echo "error: python runtime not found for scripted QEMU input"
    echo "Set PYTHON_BIN to a Python 3 executable path."
    exit 1
fi

if [ -n "${QEMU_DEBUG:-}" ]; then
    QEMU_DEBUG_FLAGS=(-d "${QEMU_DEBUG}")
fi

QEMU_ARGS=(
    -machine "$MACHINE"
    -cpu "$CPU"
    -m "$MEM_MB"
    -serial stdio
    -display none
    -no-reboot
    "${QEMU_DEBUG_FLAGS[@]}"
    -device isa-debug-exit,iobase=0xf4,iosize=0x04
    -cdrom "$ISO"
)

if [ -n "$DISK_PATH" ]; then
    if [ ! -f "$DISK_PATH" ]; then
        mkdir -p "$(dirname "$DISK_PATH")"
        dd if=/dev/zero of="$DISK_PATH" bs=1M count=1 status=none
    fi
    QEMU_ARGS+=(
        -drive "file=$DISK_PATH,format=raw,if=none,id=disk0"
        -device "$BLOCK_DEVICE"
    )
fi

if [ "$WITH_NET" -eq 1 ]; then
    QEMU_ARGS+=(
        -netdev user,id=n0
        -device "$NET_DEVICE"
    )
fi

# QEMU invocation from MILESTONES.md section 3.
# Adjust OVMF paths or remove pflash lines for BIOS-mode Limine boot.
set +e
if [ -n "$STDIN_FILE" ]; then
    "${PYTHON_RUNNER}" tools/qemu_session_runner_v1.py \
        --stdin-file "$STDIN_FILE" \
        --marker "GOSH: session ready" \
        -- \
        "${QEMU_BIN}" "${QEMU_ARGS[@]}"
    status=$?
else
    "${QEMU_BIN}" "${QEMU_ARGS[@]}"
    status=$?
fi
set -e

if [ "$status" -eq 0 ] || [ "$status" -eq 1 ] || [ "$status" -eq "$QEMU_SUCCESS_EXIT" ]; then
    exit 0
fi

exit "$status"
