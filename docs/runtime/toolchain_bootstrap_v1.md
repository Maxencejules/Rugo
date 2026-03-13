# Runtime Toolchain Bootstrap v1

Date: 2026-03-04  
Milestone: M11 Runtime + Toolchain Maturity v1

## Purpose

Document the deterministic setup and validation flow for the runtime/toolchain
lane used by supported `GOOS=rugo` stock-Go artifacts.

## Prerequisites

- `bash`
- `python3`
- `go` (stock toolchain; CI uses Go `1.25.3`)
- existing repo prerequisites from `docs/BUILD.md`

## Bootstrap check

```bash
bash tools/bootstrap_go_port_v1.sh --check
```

Expected output includes:

- resolved Go tool version,
- required runtime docs/tools presence,
- success marker `runtime-bootstrap: ok`.

## Supported build path

```bash
make build-go-std
make image-std
make smoke-std
```

`make build-go-std` rebuilds the stock-Go payload through
`tools/bootstrap_go_port_v1.sh --rebuild` and emits:

- `out/gostd.bin`
- `out/gostd-contract.env`
- `out/runtime-toolchain-contract.env`

## Contract artifact emission

```bash
python3 tools/runtime_toolchain_contract_v1.py \
  --out out/runtime-toolchain-contract.env
```

Output:

- `out/runtime-toolchain-contract.env`

## Reproducibility check

```bash
python3 tools/runtime_toolchain_contract_v1.py \
  --repro \
  --out out/runtime-toolchain-repro.json
```

Output:

- `out/runtime-toolchain-repro.json`
- non-zero exit code only if artifact hashes differ across two rebuilds

## Full M11 local gate

```bash
make test-runtime-maturity
```
