#!/usr/bin/env bash
# Boot an ISO in QEMU and assert ordered serial markers without Python.
set -euo pipefail

ISO=""
LABEL="boot"
EXPECT=()

while [ "$#" -gt 0 ]; do
    case "$1" in
        --iso)
            ISO="${2:-}"
            shift 2
            ;;
        --label)
            LABEL="${2:-}"
            shift 2
            ;;
        --expect)
            EXPECT+=("${2:-}")
            shift 2
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            echo "usage: smoke_boot.sh --iso PATH [--label NAME] --expect MARKER..." >&2
            exit 1
            ;;
    esac
done

if [ -z "$ISO" ]; then
    echo "error: --iso is required" >&2
    exit 1
fi

if [ "${#EXPECT[@]}" -eq 0 ]; then
    echo "error: at least one --expect marker is required" >&2
    exit 1
fi

TMP_LOG="$(mktemp)"
trap 'rm -f "$TMP_LOG"' EXIT

if ! ./tools/run_qemu.sh --iso "$ISO" >"$TMP_LOG" 2>&1; then
    cat "$TMP_LOG"
    echo "smoke-$LABEL: QEMU boot failed" >&2
    exit 1
fi

last_line=0
for marker in "${EXPECT[@]}"; do
    line="$(grep -Fn "$marker" "$TMP_LOG" | head -n 1 | cut -d: -f1 || true)"
    if [ -z "$line" ]; then
        cat "$TMP_LOG"
        echo "smoke-$LABEL: missing marker: $marker" >&2
        exit 1
    fi
    if [ "$line" -lt "$last_line" ]; then
        cat "$TMP_LOG"
        echo "smoke-$LABEL: marker out of order: $marker" >&2
        exit 1
    fi
    last_line="$line"
done

cat "$TMP_LOG"
echo "smoke-$LABEL: ok"
