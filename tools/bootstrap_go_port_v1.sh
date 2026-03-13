#!/usr/bin/env bash
# M11 runtime/toolchain bootstrap checker.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:---check}"

usage() {
    cat <<'USAGE'
Usage:
  bash tools/bootstrap_go_port_v1.sh --check
  bash tools/bootstrap_go_port_v1.sh --rebuild

Modes:
  --check    Validate toolchain/runtime prerequisites and required files.
  --rebuild  Run --check and rebuild the stock-Go runtime artifact.
USAGE
}

require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "runtime-bootstrap: missing required command: $cmd" >&2
        exit 1
    fi
}

require_file() {
    local rel="$1"
    if [[ ! -f "$ROOT/$rel" ]]; then
        echo "runtime-bootstrap: missing required file: $rel" >&2
        exit 1
    fi
}

parse_go_minor() {
    local ver="$1"
    # Accept values like: go1.25.3, go1.25rc1
    if [[ "$ver" =~ ^go([0-9]+)\.([0-9]+) ]]; then
        echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}"
        return 0
    fi
    return 1
}

case "$MODE" in
    --check|--rebuild) ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        usage >&2
        exit 2
        ;;
esac

require_cmd bash
require_cmd python3
require_cmd go

GO_VERSION="$(go env GOVERSION 2>/dev/null || true)"
if [[ -z "$GO_VERSION" ]]; then
    GO_VERSION="$(go version | awk '{print $3}')"
fi

if ! parsed="$(parse_go_minor "$GO_VERSION")"; then
    echo "runtime-bootstrap: unable to parse go version: $GO_VERSION" >&2
    exit 1
fi

GO_MAJOR="$(awk '{print $1}' <<<"$parsed")"
GO_MINOR="$(awk '{print $2}' <<<"$parsed")"
if (( GO_MAJOR < 1 || (GO_MAJOR == 1 && GO_MINOR < 22) )); then
    echo "runtime-bootstrap: go version too old: $GO_VERSION (need >= go1.22)" >&2
    exit 1
fi

require_file "tools/build_go_std_spike.sh"
require_file "tools/gostd_stock_builder/main.go"
require_file "tools/runtime_toolchain_contract_v1.py"
require_file "tools/extract_kernel_syscalls.py"
require_file "tools/extract_go_std_syscalls.py"
require_file "docs/runtime/port_contract_v1.md"
require_file "docs/runtime/syscall_coverage_matrix_v1.md"
require_file "docs/runtime/abi_stability_policy_v1.md"
require_file "docs/runtime/toolchain_bootstrap_v1.md"
require_file "docs/runtime/maintainers_v1.md"
require_file "docs/M11_EXECUTION_BACKLOG.md"

echo "runtime-bootstrap: go=$GO_VERSION"
echo "runtime-bootstrap: root=$ROOT"

if [[ "$MODE" == "--rebuild" ]]; then
    echo "runtime-bootstrap: rebuilding stock-Go artifact..."
    (
        cd "$ROOT"
        bash tools/build_go_std_spike.sh
    )
fi

echo "runtime-bootstrap: ok"
