#!/usr/bin/env bash
# Build G2 stock-Go artifact contract.
# Produces out/gostd.bin and out/gostd-contract.env.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/out"
mkdir -p "$OUT"
GO_CACHE="$OUT/go-build-cache"
GO_TMP="$OUT/go-tmp"
mkdir -p "$GO_CACHE" "$GO_TMP"

prepend_path_if_dir() {
    if [ -d "$1" ]; then
        PATH="$1:$PATH"
    fi
}

# Support both MSYS (/c/...) and WSL (/mnt/c/...) path styles.
prepend_path_if_dir "/c/Progra~1/Go/bin"
prepend_path_if_dir "/c/Program Files/Go/bin"
prepend_path_if_dir "/mnt/c/Progra~1/Go/bin"
prepend_path_if_dir "/mnt/c/Program Files/Go/bin"
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

GO_BIN="$(find_tool go.exe go /c/Progra~1/Go/bin/go.exe /mnt/c/Progra~1/Go/bin/go.exe)" || {
    echo "ERROR: go not found in PATH"
    exit 1
}

echo "==> Using go tool: $GO_BIN"

echo "==> Building Go std stock artifact with Go toolchain..."
(
    cd "$ROOT"
    if [[ "$GO_BIN" == *.exe ]]; then
        POWERSHELL_BIN="$(find_tool powershell.exe /c/WINDOWS/System32/WindowsPowerShell/v1.0/powershell.exe /mnt/c/WINDOWS/System32/WindowsPowerShell/v1.0/powershell.exe)" || {
            echo "ERROR: powershell.exe not found in PATH"
            exit 1
        }
        GO_HOST="$(to_host_path "$GO_BIN")"
        ROOT_HOST="$(to_host_path "$ROOT")"
        GO_CACHE_HOST="$(to_host_path "$GO_CACHE")"
        GO_TMP_HOST="$(to_host_path "$GO_TMP")"
        "$POWERSHELL_BIN" -NoProfile -Command "\$env:CC=''; \$env:CXX=''; \$env:GOENV='off'; \$env:CGO_ENABLED='0'; \$env:GOCACHE='$GO_CACHE_HOST'; \$env:GOTMPDIR='$GO_TMP_HOST'; Set-Location -LiteralPath '$ROOT_HOST'; & '$GO_HOST' run .\\tools\\gostd_stock_builder\\main.go"
    else
        GOENV=off CGO_ENABLED=0 GOCACHE="$GO_CACHE" GOTMPDIR="$GO_TMP" "$GO_BIN" run ./tools/gostd_stock_builder/main.go
    fi
)

BIN_SIZE=$(stat -c%s "$OUT/gostd.bin" 2>/dev/null || stat -f%z "$OUT/gostd.bin" 2>/dev/null || wc -c < "$OUT/gostd.bin")
echo "==> Go std stock binary: $OUT/gostd.bin ($BIN_SIZE bytes)"
echo "==> Contract file: $OUT/gostd-contract.env"
