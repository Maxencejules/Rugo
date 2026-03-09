# Kernel Reliability Model v1

Date: 2026-03-09  
Milestone: M22 Kernel Reliability + Soak v1  
Reliability Model ID: `rugo.kernel_reliability_model.v1`  
Fault Matrix ID: `rugo.kernel_fault_matrix.v1`

## Purpose

Define release-blocking reliability expectations for long-running kernel stress
and deterministic fault-injection campaigns.

## Reliability targets (v1)

Soak duration target: 24 hours.

Minimum simulated campaign length in CI: 1440 iterations (1 iteration = 1 minute).

Maximum allowed kernel panics: 0.

Maximum allowed watchdog resets: 0.

Maximum allowed deadlock events: 0.

Maximum allowed data-corruption events: 0.

## Soak workload mix

The soak lane must continuously mix workload classes that exercise core kernel
paths:

- `syscall_spam`
- `ipc_loop`
- `blk_loop`
- `pressure_shm`
- `thread_spawn`
- `vm_map`

## Fault matrix baseline

Fault classes in v1 campaign:

- `irq_storm`
- `scheduler_starvation`
- `allocator_pressure`
- `ipc_queue_saturation`
- `virtio_retry_timeout`
- `timer_drift_burst`

The v1 baseline requires deterministic recovery from all injected faults in the
declared matrix.

## Artifacts and enforcement

- Soak artifact tool: `tools/run_kernel_soak_v1.py`
- Fault campaign tool: `tools/run_fault_campaign_kernel_v1.py`
- Local gate: `make test-kernel-reliability-v1`
- CI gate: `Kernel reliability v1 gate`

Required M22 checks:

- `tests/stress/test_kernel_soak_24h_v1.py`
- `tests/stress/test_fault_injection_matrix_v1.py`
- `tests/stress/test_reliability_artifact_schema_v1.py`
- `tests/stress/test_kernel_reliability_gate_v1.py`

