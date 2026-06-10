# Default Alpha Base OS v1

Date: 2026-03-20  
Status: active

This document defines the bounded Alpha claim for the default Rust-kernel plus
Go-userspace lane.

It is intentionally narrower than the broader desktop-oriented `v1` product
definition in [../RUGO_V1_PRODUCT.md](../RUGO_V1_PRODUCT.md).

## 1. Alpha Definition

### Declared target

- Image: `out/os-go.iso`
- Kernel: `out/kernel-go.elf`
- Machine: `q35`
- CPU: `qemu64`
- Storage: `virtio-blk-pci,drive=disk0,disable-modern=on`
- Network: `virtio-net-pci,netdev=n0,disable-modern=on`
- User surface: serial shell on `-serial stdio`

### Supported

- deterministic boot from Limine to `goinit -> gosvcm -> timesvc -> diagsvc/pkgsvc -> shell`
- one live shell session on the declared QEMU target
- persistent runtime state on the attached virtio block disk
- explicit package sync plus bounded bundled-app execution through shell commands
- readable serial logs for boot, service state, package flow, storage replay,
  network checks, and ordered shutdown
- diagnosable service failure with bounded restart when the shell is told to
  `crash`

### Not supported

- graphical desktop claims on this Alpha definition
- alternate hardware targets
- Wi-Fi, GPU breadth, or native NVMe as part of this default-lane Alpha
- broad external app compatibility claims
- unattended installer, update, or recovery UX on this shell-first target

## 2. Gap Analysis

The default lane was close to Alpha mechanically, but not coherently:

- `boot-demo` booted the ISO without the persistent disk or wired net needed by
  the real runtime path.
- the shell was a scripted self-test actor that exited on its own rather than a
  usable session.
- restart proof existed only because the shell intentionally failed during
  normal boot.
- validation of storage, package flow, and network relied on dedicated pytest
  fixtures more than the public `boot-demo` path.
- the repo had a broader `product alpha` story for the native desktop lane,
  which did not describe the default shell lane honestly.

## 3. Consolidation Plan

- make `boot-demo` launch the declared q35 shell target with disk and net
- keep one canonical boot graph and move subsystem exercise behind explicit
  shell commands instead of hidden boot-time demo logic
- preserve the existing service manager, restart policy, diagnostics, and
  persistence primitives
- make smoke and runtime tooling feed the shell an explicit command script for
  `health` and `shutdown`
- keep the Alpha target shell-first and minimal instead of expanding scope

## 4. Updated Boot And Service Flow

Phase order:
`kernel -> goinit bootstrap -> gosvcm core -> gosvcm base -> operational -> shell session -> ordered shutdown`

Detailed flow:

1. The kernel boots, probes the declared block and net devices, and exposes the
   existing syscall/runtime surface.
2. `goinit` validates the manifest and starts `gosvcm`.
3. `gosvcm` deterministically launches:
   - `timesvc` in `core`
   - `diagsvc` and optional `pkgsvc` in `base`
   - `shell` in `session`
4. The shell reaches `ready`, prints a prompt, and stays alive until the user
   types `shutdown` or `exit`.
5. The shell's explicit commands drive runtime validation and normal operator
   actions:
   - `health`
   - `status`
   - `time`
   - `storage`
   - `netcheck`
   - `pkg`
   - `apps`
   - `run base-shell`
   - `run net-tools`
   - `run media-suite`
   - `crash`
   - `shutdown`
6. After `shutdown`, `gosvcm` performs reverse ordered teardown and `goinit`
   emits the final result.

## 5. Validation Steps

### Build and boot

```bash
make image-demo
make boot-demo
```

For scripted validation on the declared q35 target:

```bash
python tools/qemu_session_runner_v1.py --stdin-file validation/default-alpha-shell.txt --marker "GOSH: session ready" -- "C:\Program Files\qemu\qemu-system-x86_64.exe" -machine q35 -cpu qemu64 -m 1024 -serial stdio -display none -no-reboot -device isa-debug-exit,iobase=0xf4,iosize=0x04 -cdrom out/os-go.iso -drive file=out/os-go-alpha.img,format=raw,if=none,id=disk0 -device virtio-blk-pci,drive=disk0,disable-modern=on -netdev user,id=n0 -device virtio-net-pci,netdev=n0,disable-modern=on
```

On the current Windows host, that scripted helper is the reliable automation path
for `boot-demo`; the interactive `make boot-demo` target remains the supported
manual operator entrypoint.

### Smoke path

```bash
make smoke-demo
```

### Persistence across reboot cycles

```bash
python tools/collect_booted_runtime_v1.py --image out/os-go.iso --kernel out/kernel-go.elf --out out/booted-runtime-v1.json
python -m pytest tests/pkg/test_default_shell_app_runtime_v1.py -v
```

### Package install and run path

```bash
python -m pytest tests/pkg/test_default_shell_app_runtime_v1.py -v
```

### Service restart or failure behavior

```bash
python -m pytest tests/runtime/test_process_scheduler_runtime_v2.py -v
```

### Logging and crash evidence visibility

```bash
python tools/collect_booted_runtime_v1.py --image out/os-go.iso --kernel out/kernel-go.elf --out out/booted-runtime-v1.json
python -m pytest tests/runtime/test_service_control_runtime_v1.py -v
```

### Full minimum Alpha checklist

```bash
make image-demo
make boot-demo
make smoke-demo
python tools/collect_booted_runtime_v1.py --image out/os-go.iso --kernel out/kernel-go.elf --out out/booted-runtime-v1.json
python -m pytest tests/go/test_go_user_service.py tests/runtime/test_service_boot_runtime_v2.py tests/runtime/test_service_control_runtime_v1.py tests/runtime/test_process_scheduler_runtime_v2.py tests/pkg/test_default_shell_app_runtime_v1.py -v
```

## 6. Limitations And Explicit Non-Goals

- The default Alpha surface is serial-shell-first, not desktop-first.
- Bundled app execution is intentionally tiny and state-driven; it is not a
  general package ecosystem promise.
- Recovery is diagnosable through logs and crash evidence, but not yet exposed
  as a richer end-user workflow on this target.
- The support promise is only the declared q35 virtio shell lane.
- Desktop, native NVMe, installer UX, and broader product qualification remain
  separate tracks and must not be projected onto this Alpha claim automatically.
