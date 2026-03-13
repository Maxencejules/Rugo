# Runtime Port Contract v1

Date: 2026-03-04  
Milestone: M11 Runtime + Toolchain Maturity v1  
Status: active release gate

## Purpose

Define the maintainer-grade runtime/toolchain contract for the Rugo supported
stock-Go lane so releases do not depend on implicit assumptions from the G2
spike.

## Target contract

- Target GOOS: `rugo`
- Target GOARCH: `amd64`
- Syscall invocation ABI: `int 0x80` with register contract from
  `docs/abi/syscall_v0.md` and `docs/abi/syscall_v1.md`
- Runtime binary artifact: `out/gostd.bin`
- Runtime contract artifact: `out/gostd-contract.env`
- Supported build path:
  - `make build-go-std`
  - `make image-std`
  - `make boot-std`
  - `make smoke-std`

## Runtime behavior contract (v1 lane)

- Runtime-facing syscall failures return deterministic `-1`.
- Thread spawn/exit behavior follows `docs/abi/process_thread_model_v1.md`.
- Runtime time/yield/vm bridge markers must stay regression-gated by tests.
- Compatibility and security controls from M8/M10 remain active constraints on
  runtime behavior.

## Toolchain contract (v1 lane)

- Stock Go builder path remains:
  - `tools/build_go_std_spike.sh`
  - `tools/gostd_stock_builder/main.go`
- Toolchain reproducibility artifacts:
  - `out/runtime-toolchain-contract.env`
  - `out/runtime-toolchain-repro.json`
- Source-of-truth interface artifact:
  - `out/gostd-syscall-interface.json`
- Bootstrap check entrypoint:
  - `tools/bootstrap_go_port_v1.sh --check`

## Required release gates

- Local gate: `make test-runtime-maturity`
- CI gate: `Runtime + toolchain maturity v1 gate`
- Contract tests:
  - `tests/runtime/test_runtime_contract_docs_v1.py`
  - `tests/runtime/test_runtime_abi_window_v1.py`
  - `tests/runtime/test_runtime_stress_v1.py`
