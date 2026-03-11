# Rugo - Top-level Makefile
# Default target: Rust M0 kernel
# Legacy C kernel: make -C legacy build
#
# Use bash recipes everywhere so inline env assignments (for mkimage variants)
# work consistently on Windows and Unix.
ifeq ($(OS),Windows_NT)
SHELL := C:/WINDOWS/system32/bash.exe
.SHELLFLAGS := -lc
else
SHELL := bash
endif

.PHONY: build image build-panic image-panic build-pf image-pf build-idt image-idt \
       build-sched image-sched \
       build-user-hello image-user-hello build-syscall image-syscall \
       build-thread-exit image-thread-exit \
       build-thread-spawn image-thread-spawn \
       build-vm-map image-vm-map \
       build-syscall-invalid image-syscall-invalid \
       build-stress-syscall image-stress-syscall \
       build-stress-ipc image-stress-ipc \
       build-stress-blk image-stress-blk \
       build-pressure-shm image-pressure-shm \
       build-yield image-yield \
       build-user-fault image-user-fault \
       build-ipc image-ipc build-ipc-badptr-send image-ipc-badptr-send \
       build-ipc-badptr-recv image-ipc-badptr-recv \
       build-svc-badptr image-svc-badptr \
       build-ipc-badptr-svc image-ipc-badptr-svc \
       build-ipc-buffer-full image-ipc-buffer-full \
       build-ipc-waiter-busy image-ipc-waiter-busy \
       build-ipc-svc-overwrite image-ipc-svc-overwrite \
       build-svc-full image-svc-full \
       build-svc-bad-endpoint image-svc-bad-endpoint \
       build-shm image-shm \
       build-quota-endpoints image-quota-endpoints \
       build-quota-shm image-quota-shm \
       build-quota-threads image-quota-threads \
       build-blk image-blk \
       build-blk-badlen image-blk-badlen \
       build-blk-badptr image-blk-badptr \
       build-blk-invariants image-blk-invariants \
       build-blk-init-fail image-blk-init-fail \
       build-fs image-fs \
       build-fs-badmagic image-fs-badmagic \
       build-pkg-hash image-pkg-hash \
       build-net image-net \
       build-go image-go \
       build-go-std image-go-std \
       build-sec-rights image-sec-rights \
       build-sec-filter image-sec-filter \
       test-security-baseline test-runtime-maturity test-process-scheduler-v2 test-compat-v2 test-network-stack-v1 test-network-stack-v2 \
       test-storage-reliability-v1 test-storage-reliability-v2 test-release-engineering-v1 test-release-ops-v2 test-abi-stability-v3 test-kernel-reliability-v1 \
       test-firmware-attestation-v1 test-perf-regression-v1 test-userspace-model-v2 test-pkg-ecosystem-v3 test-update-trust-v1 test-app-compat-v3 test-security-hardening-v3 test-vuln-response-v1 \
       test-observability-v2 test-crash-dump-v1 test-ops-ux-v3 test-release-lifecycle-v2 test-supply-chain-revalidation-v1 test-conformance-v1 test-fleet-ops-v1 test-fleet-rollout-safety-v1 test-maturity-qual-v1 test-desktop-stack-v1 test-gui-app-compat-v1 \
       test-compat-surface-v1 test-posix-gap-closure-v1 test-hw-matrix-v4 test-hw-baremetal-promotion-v1 test-storage-platform-v1 test-storage-feature-contract-v1 test-ecosystem-scale-v1 test-app-catalog-health-v1 \
       test-evidence-integrity-v1 test-synthetic-evidence-ban-v1 test-process-readiness-parity-v1 test-posix-gap-closure-v2 test-isolation-baseline-v1 test-namespace-cgroup-v1 \
       test-hw-firmware-smp-v1 test-native-driver-matrix-v1 test-hw-matrix-v6 test-virtio-platform-v1 test-baremetal-io-baseline-v1 test-usb-input-removable-v1 test-hw-claim-promotion-v1 test-hw-support-tier-audit-v1 test-display-runtime-v1 test-scanout-path-v1 test-input-seat-v1 test-hid-event-path-v1 test-window-system-v1 test-compositor-damage-v1 test-real-ecosystem-desktop-v2 test-real-app-catalog-v2 \
       run test-qemu test-hw-matrix test-hw-matrix-v2 test-hw-matrix-v3 test-hw-matrix-v4 repro-check clean legacy docker-all docker-legacy

# Tools
NASM    ?= nasm
LD      ?= ld
CC      ?= cc
XORRISO ?= xorriso
PYTHON  ?= python3
WSL_PATH ?= /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/mnt/c/mingw64/mingw64/bin
ifeq ($(OS),Windows_NT)
SUBMAKE ?= /mnt/c/mingw64/mingw64/bin/mingw32-make.exe
else
SUBMAKE ?= $(MAKE)
endif

ifeq ($(OS),Windows_NT)
RUSTUP_TOOLCHAIN ?= nightly-x86_64-pc-windows-gnu
export RUSTUP_TOOLCHAIN
export PATH := /usr/bin:/bin:/usr/sbin:/sbin:/mnt/c/Users/$(USERNAME)/.cargo/bin:/mnt/c/mingw64/mingw64/bin
NASM    := /mnt/c/mingw64/mingw64/bin/nasm.exe
CARGO   ?= /mnt/c/Users/$(USERNAME)/.cargo/bin/cargo.exe
PYTHON  := /mnt/c/Users/$(USERNAME)/AppData/Local/Programs/Python/Python312/python.exe
CC      := /mnt/c/mingw64/mingw64/bin/gcc.exe
LD      := /mnt/c/Users/$(USERNAME)/.rustup/toolchains/$(RUSTUP_TOOLCHAIN)/lib/rustlib/x86_64-pc-windows-gnu/bin/rust-lld.exe -flavor gnu -m elf_x86_64
else
CARGO   ?= cargo
endif

export CC XORRISO

# Flags
NASMFLAGS = -f elf64
LDFLAGS   = -nostdlib -static -T boot/linker.ld

# Output
OUT = out

# Rust kernel staticlib
CARGO_TARGET = x86_64-unknown-none
KERNEL_LIB   = kernel_rs/target/$(CARGO_TARGET)/release/librugo_kernel.a

# --- Targets ------------------------------------------------------------------

build: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release
	$(LD) $(LDFLAGS) -o $(OUT)/kernel.elf $(ASM_OBJS) $(KERNEL_LIB)

$(OUT):
	mkdir -p $(OUT)

# Assembly objects
ASM_OBJS = $(OUT)/entry.o $(OUT)/isr.o $(OUT)/context.o

# --- Assembly -----------------------------------------------------------------

$(OUT)/entry.o: arch/x86_64/entry.asm | $(OUT)
	$(NASM) $(NASMFLAGS) $< -o $@

$(OUT)/isr.o: arch/x86_64/isr.asm | $(OUT)
	$(NASM) $(NASMFLAGS) $< -o $@

$(OUT)/context.o: arch/x86_64/context.asm | $(OUT)
	$(NASM) $(NASMFLAGS) $< -o $@

# --- Rust kernel --------------------------------------------------------------

$(KERNEL_LIB): kernel_rs/src/lib.rs kernel_rs/Cargo.toml kernel_rs/.cargo/config.toml
	cd kernel_rs && $(CARGO) build --release

# --- Link ---------------------------------------------------------------------

# --- Panic-test kernel (feature-gated) ----------------------------------------

build-panic: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features panic_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-panic.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- Page-fault-test kernel ---------------------------------------------------

build-pf: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features pf_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-pf.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- IDT-smoke-test kernel ---------------------------------------------------

build-idt: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features idt_smoke_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-idt.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- Scheduler-test kernel ----------------------------------------------------

build-sched: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features sched_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-sched.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: User-hello-test kernel -----------------------------------------------

build-user-hello: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features user_hello_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-user-hello.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: Syscall-test kernel --------------------------------------------------

build-syscall: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features syscall_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-syscall.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: Thread-exit-test kernel ----------------------------------------------

build-thread-exit: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features thread_exit_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-thread-exit.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: Thread-spawn-test kernel ---------------------------------------------

build-thread-spawn: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features thread_spawn_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-thread-spawn.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: VM-map-test kernel ---------------------------------------------------

build-vm-map: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features vm_map_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-vm-map.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: Invalid-syscall-test kernel ------------------------------------------

build-syscall-invalid: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features syscall_invalid_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-syscall-invalid.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: Stress-syscall-test kernel -------------------------------------------

build-stress-syscall: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features stress_syscall_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-stress-syscall.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: Stress-IPC-test kernel ----------------------------------------------

build-stress-ipc: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features stress_ipc_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-stress-ipc.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M5: Stress-blk-test kernel ----------------------------------------------

build-stress-blk: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features stress_blk_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-stress-blk.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: Pressure-SHM-test kernel --------------------------------------------

build-pressure-shm: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features pressure_shm_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-pressure-shm.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: Yield-test kernel ----------------------------------------------------

build-yield: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features yield_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-yield.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M3: User-fault-test kernel -----------------------------------------------

build-user-fault: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features user_fault_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-user-fault.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: IPC ping-pong test kernel -------------------------------------------

build-ipc: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features ipc_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-ipc.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: IPC bad-pointer send test kernel ------------------------------------

build-ipc-badptr-send: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features ipc_badptr_send_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-ipc-badptr-send.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: IPC bad-pointer recv test kernel ------------------------------------

build-ipc-badptr-recv: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features ipc_badptr_recv_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-ipc-badptr-recv.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: Service registry bad-pointer test kernel -----------------------------

build-svc-badptr: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features svc_badptr_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-svc-badptr.elf $(ASM_OBJS) $(KERNEL_LIB)

# Backward-compatible alias
build-ipc-badptr-svc: build-svc-badptr

# --- R4: IPC buffer-full test kernel ------------------------------------------

build-ipc-buffer-full: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features ipc_buffer_full_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-ipc-buffer-full.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: IPC waiter-busy semantics test kernel --------------------------------

build-ipc-waiter-busy: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features ipc_waiter_busy_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-ipc-waiter-busy.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: SVC overwrite test kernel -------------------------------------------

build-ipc-svc-overwrite: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features svc_overwrite_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-ipc-svc-overwrite.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: SVC table-full test kernel ------------------------------------------

build-svc-full: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features svc_full_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-svc-full.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: SVC invalid-endpoint test kernel -------------------------------------

build-svc-bad-endpoint: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features svc_bad_endpoint_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-svc-bad-endpoint.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: SHM bulk test kernel ------------------------------------------------

build-shm: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features shm_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-shm.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: Quota endpoint test kernel ------------------------------------------

build-quota-endpoints: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features quota_endpoints_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-quota-endpoints.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: Quota SHM test kernel ------------------------------------------------

build-quota-shm: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features quota_shm_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-quota-shm.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- R4: Quota thread test kernel ---------------------------------------------

build-quota-threads: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features quota_threads_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-quota-threads.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M5: VirtIO block test kernel ---------------------------------------------

build-blk: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features blk_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-blk.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M5: VirtIO block bad-length test kernel ----------------------------------

build-blk-badlen: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features blk_badlen_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-blk-badlen.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- M5: VirtIO block bad-pointer test kernel ---------------------------------

build-blk-badptr: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features blk_badptr_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-blk-badptr.elf $(ASM_OBJS) $(KERNEL_LIB)

# --- Image / Run / Test -------------------------------------------------------

image: build
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" bash tools/mkimage.sh

image-panic: build-panic
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-panic.elf ISO_NAME=os-panic.iso bash tools/mkimage.sh

image-pf: build-pf
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-pf.elf ISO_NAME=os-pf.iso bash tools/mkimage.sh

image-idt: build-idt
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-idt.elf ISO_NAME=os-idt.iso bash tools/mkimage.sh

image-sched: build-sched
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-sched.elf ISO_NAME=os-sched.iso bash tools/mkimage.sh

image-user-hello: build-user-hello
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-user-hello.elf ISO_NAME=os-user-hello.iso bash tools/mkimage.sh

image-syscall: build-syscall
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-syscall.elf ISO_NAME=os-syscall.iso bash tools/mkimage.sh

image-thread-exit: build-thread-exit
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-thread-exit.elf ISO_NAME=os-thread-exit.iso bash tools/mkimage.sh

image-thread-spawn: build-thread-spawn
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-thread-spawn.elf ISO_NAME=os-thread-spawn.iso bash tools/mkimage.sh

image-vm-map: build-vm-map
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-vm-map.elf ISO_NAME=os-vm-map.iso bash tools/mkimage.sh

image-syscall-invalid: build-syscall-invalid
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-syscall-invalid.elf ISO_NAME=os-syscall-invalid.iso bash tools/mkimage.sh

image-stress-syscall: build-stress-syscall
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-stress-syscall.elf ISO_NAME=os-stress-syscall.iso bash tools/mkimage.sh

image-stress-ipc: build-stress-ipc
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-stress-ipc.elf ISO_NAME=os-stress-ipc.iso bash tools/mkimage.sh

image-stress-blk: build-stress-blk
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-stress-blk.elf ISO_NAME=os-stress-blk.iso bash tools/mkimage.sh

image-pressure-shm: build-pressure-shm
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-pressure-shm.elf ISO_NAME=os-pressure-shm.iso bash tools/mkimage.sh

image-yield: build-yield
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-yield.elf ISO_NAME=os-yield.iso bash tools/mkimage.sh

image-user-fault: build-user-fault
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-user-fault.elf ISO_NAME=os-user-fault.iso bash tools/mkimage.sh

image-ipc: build-ipc
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-ipc.elf ISO_NAME=os-ipc.iso bash tools/mkimage.sh

image-shm: build-shm
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-shm.elf ISO_NAME=os-shm.iso bash tools/mkimage.sh

image-quota-endpoints: build-quota-endpoints
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-quota-endpoints.elf ISO_NAME=os-quota-endpoints.iso bash tools/mkimage.sh

image-quota-shm: build-quota-shm
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-quota-shm.elf ISO_NAME=os-quota-shm.iso bash tools/mkimage.sh

image-quota-threads: build-quota-threads
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-quota-threads.elf ISO_NAME=os-quota-threads.iso bash tools/mkimage.sh

image-ipc-badptr-send: build-ipc-badptr-send
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-ipc-badptr-send.elf ISO_NAME=os-ipc-badptr-send.iso bash tools/mkimage.sh

image-ipc-badptr-recv: build-ipc-badptr-recv
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-ipc-badptr-recv.elf ISO_NAME=os-ipc-badptr-recv.iso bash tools/mkimage.sh

image-svc-badptr: build-svc-badptr
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-svc-badptr.elf ISO_NAME=os-svc-badptr.iso bash tools/mkimage.sh

# Backward-compatible alias
image-ipc-badptr-svc: build-svc-badptr
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-svc-badptr.elf ISO_NAME=os-ipc-badptr-svc.iso bash tools/mkimage.sh

image-ipc-buffer-full: build-ipc-buffer-full
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-ipc-buffer-full.elf ISO_NAME=os-ipc-buffer-full.iso bash tools/mkimage.sh

image-ipc-waiter-busy: build-ipc-waiter-busy
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-ipc-waiter-busy.elf ISO_NAME=os-ipc-waiter-busy.iso bash tools/mkimage.sh

image-ipc-svc-overwrite: build-ipc-svc-overwrite
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-ipc-svc-overwrite.elf ISO_NAME=os-svc-overwrite.iso bash tools/mkimage.sh

image-svc-full: build-svc-full
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-svc-full.elf ISO_NAME=os-svc-full.iso bash tools/mkimage.sh

image-svc-bad-endpoint: build-svc-bad-endpoint
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-svc-bad-endpoint.elf ISO_NAME=os-svc-bad-endpoint.iso bash tools/mkimage.sh

image-blk: build-blk
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-blk.elf ISO_NAME=os-blk.iso bash tools/mkimage.sh

image-blk-badlen: build-blk-badlen
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-blk-badlen.elf ISO_NAME=os-blk-badlen.iso bash tools/mkimage.sh

image-blk-badptr: build-blk-badptr
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-blk-badptr.elf ISO_NAME=os-blk-badptr.iso bash tools/mkimage.sh

# --- M5: VirtIO block init invariants test kernel -----------------------------

build-blk-invariants: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features blk_invariants_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-blk-invariants.elf $(ASM_OBJS) $(KERNEL_LIB)

image-blk-invariants: build-blk-invariants
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-blk-invariants.elf ISO_NAME=os-blk-invariants.iso bash tools/mkimage.sh

build-blk-init-fail: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features blk_init_fail_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-blk-init-fail.elf $(ASM_OBJS) $(KERNEL_LIB)

image-blk-init-fail: build-blk-init-fail
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-blk-init-fail.elf ISO_NAME=os-blk-init-fail.iso bash tools/mkimage.sh

# --- M6: Filesystem test kernel + disk image ---------------------------------

build-fs: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features fs_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-fs.elf $(ASM_OBJS) $(KERNEL_LIB)

image-fs: build-fs
	$(PYTHON) tools/mkfs.py $(OUT)/fs-test.img
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-fs.elf ISO_NAME=os-fs.iso bash tools/mkimage.sh

build-fs-badmagic: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features fs_badmagic_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-fs-badmagic.elf $(ASM_OBJS) $(KERNEL_LIB)

image-fs-badmagic: build-fs-badmagic
	$(PYTHON) tools/mkfs.py $(OUT)/fs-badmagic.img --corrupt-superblock-magic
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-fs-badmagic.elf ISO_NAME=os-fs-badmagic.iso bash tools/mkimage.sh

build-pkg-hash: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features pkg_hash_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-pkg-hash.elf $(ASM_OBJS) $(KERNEL_LIB)

image-pkg-hash: build-pkg-hash
	$(PYTHON) tools/mkfs.py $(OUT)/fs-test.img
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-pkg-hash.elf ISO_NAME=os-pkg-hash.iso bash tools/mkimage.sh

# --- M7: VirtIO net test kernel -----------------------------------------------

build-net: $(ASM_OBJS) boot/linker.ld
	cd kernel_rs && $(CARGO) build --release --features net_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-net.elf $(ASM_OBJS) $(KERNEL_LIB)

image-net: build-net
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-net.elf ISO_NAME=os-net.iso bash tools/mkimage.sh

# --- G1: TinyGo user-space test kernel ----------------------------------------

build-go: $(ASM_OBJS) boot/linker.ld
	bash tools/build_go.sh
	cd kernel_rs && $(CARGO) build --release --features go_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-go.elf $(ASM_OBJS) $(KERNEL_LIB)

image-go: build-go
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-go.elf ISO_NAME=os-go.iso bash tools/mkimage.sh

# --- G2 spike: Go std-port candidate kernel -----------------------------------

build-go-std: $(ASM_OBJS) boot/linker.ld
	bash tools/build_go_std_spike.sh
	cd kernel_rs && $(CARGO) build --release --features go_std_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-go-std.elf $(ASM_OBJS) $(KERNEL_LIB)

image-go-std: build-go-std
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-go-std.elf ISO_NAME=os-go-std.iso bash tools/mkimage.sh

# --- M10: Security rights test kernel -----------------------------------------

build-sec-rights: $(ASM_OBJS) boot/linker.ld
	$(NASM) -f bin services/security/sec_rights.asm -o $(OUT)/sec-rights.bin
	cd kernel_rs && $(CARGO) build --release --features sec_rights_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-sec-rights.elf $(ASM_OBJS) $(KERNEL_LIB)

image-sec-rights: build-sec-rights
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-sec-rights.elf ISO_NAME=os-sec-rights.iso bash tools/mkimage.sh

# --- M10: Security profile filter test kernel ---------------------------------

build-sec-filter: $(ASM_OBJS) boot/linker.ld
	$(NASM) -f bin services/security/sec_filter.asm -o $(OUT)/sec-filter.bin
	cd kernel_rs && $(CARGO) build --release --features sec_filter_test
	$(LD) $(LDFLAGS) -o $(OUT)/kernel-sec-filter.elf $(ASM_OBJS) $(KERNEL_LIB)

image-sec-filter: build-sec-filter
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" KERNEL_ELF=kernel-sec-filter.elf ISO_NAME=os-sec-filter.iso bash tools/mkimage.sh

run: image
	./tools/run_qemu.sh

test-qemu: image image-panic image-pf image-idt image-sched image-user-hello image-syscall image-thread-exit image-thread-spawn image-vm-map image-syscall-invalid image-stress-syscall image-stress-ipc image-stress-blk image-pressure-shm image-yield image-user-fault image-ipc image-ipc-badptr-send image-ipc-badptr-recv image-svc-badptr image-ipc-buffer-full image-ipc-waiter-busy image-ipc-svc-overwrite image-svc-full image-svc-bad-endpoint image-shm image-quota-endpoints image-quota-shm image-quota-threads image-blk image-blk-badlen image-blk-badptr image-blk-invariants image-blk-init-fail image-fs image-fs-badmagic image-pkg-hash image-net image-go image-go-std
	$(PYTHON) -m pytest tests/ -v

test-hw-matrix: image-blk image-blk-badlen image-blk-badptr image-net
	$(PYTHON) -m pytest tests/hw/test_hardware_matrix_v1.py tests/hw/test_probe_negative_paths_v1.py tests/hw/test_dma_safety_v1.py -v

test-hw-matrix-v2: image-blk image-blk-badlen image-blk-badptr image-net
	$(PYTHON) -m pytest tests/hw/test_hardware_matrix_v2.py tests/hw/test_probe_negative_paths_v2.py tests/hw/test_dma_iommu_policy_v2.py tests/hw/test_acpi_boot_paths_v2.py tests/hw/test_bare_metal_smoke_v2.py tests/hw/test_hw_gate_v2.py -v --junitxml=$(OUT)/pytest-hw-matrix-v2.xml

test-hw-matrix-v3: image-blk image-blk-badlen image-blk-badptr image-net
	$(PYTHON) tools/collect_hw_diagnostics_v3.py --seed 20260306 --out $(OUT)/hw-diagnostics-v3.json
	$(PYTHON) -m pytest tests/hw/test_hardware_matrix_v3.py tests/hw/test_driver_lifecycle_v3.py tests/hw/test_suspend_resume_v1.py tests/hw/test_hotplug_baseline_v1.py tests/hw/test_hw_gate_v3.py -v --junitxml=$(OUT)/pytest-hw-matrix-v3.xml

test-process-scheduler-v2: image-thread-spawn image-thread-exit image-yield image-user-fault
	$(PYTHON) -m pytest tests/sched/test_preempt_timer_quantum_v2.py tests/sched/test_priority_fairness_v2.py tests/sched/test_scheduler_soak_v2.py tests/user/test_process_wait_kill_v2.py tests/user/test_signal_delivery_v2.py tests/sched/test_scheduler_gate_v2.py -v --junitxml=$(OUT)/pytest-process-scheduler-v2.xml

test-compat-v2: image-go-std image-pkg-hash
	$(PYTHON) -m pytest tests/compat/test_abi_profile_v2_docs.py tests/compat/test_elf_loader_dynamic_v2.py tests/compat/test_posix_profile_v2.py tests/compat/test_external_apps_tier_v2.py tests/compat/test_compat_gate_v2.py -v --junitxml=$(OUT)/pytest-compat-v2.xml

test-security-baseline: image-sec-rights image-sec-filter
	$(PYTHON) -m pytest tests/security -v

test-runtime-maturity: image-go-std image-stress-syscall image-pressure-shm image-thread-spawn image-vm-map
	bash tools/bootstrap_go_port_v1.sh --check
	$(PYTHON) tools/runtime_toolchain_contract_v1.py --out $(OUT)/runtime-toolchain-contract.env
	$(PYTHON) tools/runtime_toolchain_contract_v1.py --repro --out $(OUT)/runtime-toolchain-repro.json
	$(PYTHON) -m pytest tests/runtime tests/go/test_std_go_binary.py tests/compat/test_posix_subset.py -v

test-network-stack-v1: image-net
	$(PYTHON) tools/run_net_interop_matrix_v1.py --out $(OUT)/net-interop-v1.json
	$(PYTHON) tools/run_net_soak_v1.py --out $(OUT)/net-soak-v1.json
	$(PYTHON) -m pytest tests/net -v

test-network-stack-v2: image-net
	$(PYTHON) tools/run_net_interop_matrix_v2.py --out $(OUT)/net-interop-v2.json
	$(PYTHON) tools/run_net_soak_v2.py --out $(OUT)/net-soak-v2.json
	$(PYTHON) -m pytest tests/net/test_tcp_interop_v2.py tests/net/test_ipv6_interop_v2.py tests/net/test_dns_stub_v2.py tests/net/test_network_gate_v2.py -v --junitxml=$(OUT)/pytest-network-stack-v2.xml

test-storage-reliability-v1: image-fs image-fs-badmagic
	$(PYTHON) tools/storage_recover_v1.py --check --out $(OUT)/storage-recovery-v1.json
	$(PYTHON) tools/run_storage_fault_campaign_v1.py --seed 20260304 --out $(OUT)/storage-fault-campaign-v1.json
	$(PYTHON) -m pytest tests/storage -v

test-storage-reliability-v2: image-fs image-fs-badmagic
	$(PYTHON) tools/storage_recover_v2.py --check --out $(OUT)/storage-recovery-v2.json
	$(PYTHON) tools/run_storage_powerfail_campaign_v2.py --seed 20260304 --out $(OUT)/storage-powerfail-v2.json
	$(PYTHON) -m pytest tests/storage/test_journal_recovery_v2.py tests/storage/test_powerfail_campaign_v2.py tests/storage/test_metadata_integrity_v2.py tests/storage/test_storage_gate_v2.py -v --junitxml=$(OUT)/pytest-storage-reliability-v2.xml

test-release-engineering-v1: image
	$(PYTHON) tools/release_contract_v1.py --out $(OUT)/release-contract-v1.json
	$(PYTHON) tools/update_repo_sign_v1.py --repo $(OUT)/update-repo-v1 --version 1.0.0 --build-sequence 1 --out $(OUT)/update-metadata-v1.json
	$(PYTHON) tools/update_client_verify_v1.py --repo $(OUT)/update-repo-v1 --metadata $(OUT)/update-metadata-v1.json --state $(OUT)/update-client-state-v1.json --expect-version 1.0.0
	$(PYTHON) tools/run_update_attack_suite_v1.py --seed 20260304 --out $(OUT)/update-attack-suite-v1.json
	$(PYTHON) tools/generate_sbom_v1.py --out $(OUT)/sbom-v1.spdx.json
	$(PYTHON) tools/generate_provenance_v1.py --out $(OUT)/provenance-v1.json
	$(PYTHON) tools/collect_support_bundle_v1.py --out $(OUT)/support-bundle-v1.json
	$(PYTHON) -m pytest tests/build tests/pkg/test_update_metadata_v1.py tests/pkg/test_update_rollback_protection_v1.py tests/pkg/test_update_attack_suite_v1.py -v

test-release-ops-v2: image
	$(PYTHON) tools/build_installer_v2.py --out $(OUT)/installer-v2.json
	$(PYTHON) tools/run_upgrade_recovery_drill_v2.py --out $(OUT)/upgrade-recovery-v2.json
	$(PYTHON) tools/collect_support_bundle_v2.py --artifacts $(OUT)/installer-v2.json $(OUT)/upgrade-recovery-v2.json --out $(OUT)/support-bundle-v2.json
	$(PYTHON) -m pytest tests/build/test_installer_recovery_v2.py tests/build/test_upgrade_rollback_v2.py tests/build/test_support_bundle_v2.py tests/build/test_operability_gate_v2.py -v --junitxml=$(OUT)/pytest-release-ops-v2.xml

test-abi-stability-v3:
	$(PYTHON) tools/check_abi_diff_v3.py --out $(OUT)/abi-diff-v3.json
	$(PYTHON) tools/check_syscall_compat_v3.py --diff-report $(OUT)/abi-diff-v3.json --out $(OUT)/syscall-compat-v3.json
	$(PYTHON) -m pytest tests/runtime/test_abi_docs_v3.py tests/runtime/test_abi_window_v3.py tests/runtime/test_abi_diff_gate_v3.py tests/compat/test_abi_compat_matrix_v3.py tests/runtime/test_abi_stability_gate_v3.py -v --junitxml=$(OUT)/pytest-abi-stability-v3.xml

test-kernel-reliability-v1:
	$(PYTHON) tools/run_kernel_soak_v1.py --seed 20260306 --out $(OUT)/kernel-soak-v1.json
	$(PYTHON) tools/run_fault_campaign_kernel_v1.py --seed 20260306 --out $(OUT)/kernel-fault-campaign-v1.json
	$(PYTHON) -m pytest tests/stress/test_kernel_soak_24h_v1.py tests/stress/test_fault_injection_matrix_v1.py tests/stress/test_reliability_artifact_schema_v1.py tests/stress/test_kernel_reliability_gate_v1.py -v --junitxml=$(OUT)/pytest-kernel-reliability-v1.xml

test-firmware-attestation-v1:
	$(PYTHON) tools/collect_measured_boot_report_v1.py --out $(OUT)/measured-boot-v1.json
	$(PYTHON) -m pytest tests/hw/test_firmware_resiliency_docs_v1.py tests/hw/test_measured_boot_attestation_v1.py tests/hw/test_tpm_eventlog_schema_v1.py tests/hw/test_firmware_attestation_gate_v1.py -v --junitxml=$(OUT)/pytest-firmware-attestation-v1.xml

test-perf-regression-v1:
	$(PYTHON) tools/run_perf_baseline_v1.py --seed 20260309 --out $(OUT)/perf-baseline-v1.json
	$(PYTHON) tools/check_perf_regression_v1.py --baseline $(OUT)/perf-baseline-v1.json --seed 20260309 --out $(OUT)/perf-regression-v1.json
	$(PYTHON) -m pytest tests/runtime/test_perf_budget_docs_v1.py tests/runtime/test_perf_regression_v1.py tests/runtime/test_perf_gate_v1.py -v --junitxml=$(OUT)/pytest-perf-regression-v1.xml

test-userspace-model-v2:
	$(PYTHON) -m pytest tests/runtime/test_service_model_docs_v2.py tests/runtime/test_service_lifecycle_v2.py tests/runtime/test_service_dependency_order_v2.py tests/runtime/test_restart_policy_v2.py tests/runtime/test_userspace_model_gate_v2.py -v --junitxml=$(OUT)/pytest-userspace-model-v2.xml

test-pkg-ecosystem-v3:
	$(PYTHON) tools/repo_policy_check_v3.py --out $(OUT)/repo-policy-v3.json
	$(PYTHON) tools/pkg_rebuild_verify_v3.py --seed 20260309 --out $(OUT)/pkg-rebuild-v3.json
	$(PYTHON) -m pytest tests/pkg/test_pkg_contract_docs_v3.py tests/pkg/test_pkg_rebuild_repro_v3.py tests/pkg/test_repo_policy_v3.py tests/pkg/test_pkg_ecosystem_gate_v3.py -v --junitxml=$(OUT)/pytest-pkg-ecosystem-v3.xml
	$(PYTHON) tools/check_update_trust_v1.py --out $(OUT)/update-trust-v1.json
	$(PYTHON) tools/run_update_key_rotation_drill_v1.py --out $(OUT)/update-key-rotation-drill-v1.json
	$(PYTHON) -m pytest tests/pkg/test_update_trust_docs_v1.py tests/pkg/test_update_metadata_expiry_v1.py tests/pkg/test_update_freeze_attack_v1.py tests/pkg/test_update_mix_and_match_v1.py tests/pkg/test_update_key_rotation_v1.py tests/pkg/test_update_trust_gate_v1.py -v --junitxml=$(OUT)/pytest-update-trust-v1.xml

test-update-trust-v1:
	$(PYTHON) tools/check_update_trust_v1.py --out $(OUT)/update-trust-v1.json
	$(PYTHON) tools/run_update_key_rotation_drill_v1.py --out $(OUT)/update-key-rotation-drill-v1.json
	$(PYTHON) -m pytest tests/pkg/test_update_trust_docs_v1.py tests/pkg/test_update_metadata_expiry_v1.py tests/pkg/test_update_freeze_attack_v1.py tests/pkg/test_update_mix_and_match_v1.py tests/pkg/test_update_key_rotation_v1.py tests/pkg/test_update_trust_gate_v1.py -v --junitxml=$(OUT)/pytest-update-trust-v1.xml

test-app-compat-v3:
	$(PYTHON) tools/run_app_compat_matrix_v3.py --seed 20260309 --out $(OUT)/app-compat-matrix-v3.json
	$(PYTHON) -m pytest tests/compat/test_app_tier_docs_v1.py tests/compat/test_cli_suite_v3.py tests/compat/test_runtime_suite_v3.py tests/compat/test_service_suite_v3.py tests/compat/test_app_compat_gate_v3.py -v --junitxml=$(OUT)/pytest-app-compat-v3.xml

test-security-hardening-v3:
	$(PYTHON) tools/run_security_attack_suite_v3.py --seed 20260309 --out $(OUT)/security-attack-suite-v3.json
	$(PYTHON) tools/run_security_fuzz_v2.py --seed 20260309 --iterations 1600 --cases 5 --out $(OUT)/security-fuzz-v2.json
	$(PYTHON) -m pytest tests/security/test_hardening_docs_v3.py tests/security/test_attack_suite_v3.py tests/security/test_fuzz_gate_v2.py tests/security/test_policy_enforcement_v3.py tests/security/test_security_hardening_gate_v3.py -v --junitxml=$(OUT)/pytest-security-hardening-v3.xml
	$(PYTHON) tools/security_advisory_lint_v1.py --out $(OUT)/security-advisory-lint-v1.json
	$(PYTHON) tools/security_embargo_drill_v1.py --out $(OUT)/security-embargo-drill-v1.json
	$(PYTHON) -m pytest tests/security/test_vuln_response_docs_v1.py tests/security/test_vuln_triage_sla_v1.py tests/security/test_embargo_workflow_v1.py tests/security/test_advisory_schema_v1.py tests/security/test_vuln_response_gate_v1.py -v --junitxml=$(OUT)/pytest-vuln-response-v1.xml

test-vuln-response-v1:
	$(PYTHON) tools/security_advisory_lint_v1.py --out $(OUT)/security-advisory-lint-v1.json
	$(PYTHON) tools/security_embargo_drill_v1.py --out $(OUT)/security-embargo-drill-v1.json
	$(PYTHON) -m pytest tests/security/test_vuln_response_docs_v1.py tests/security/test_vuln_triage_sla_v1.py tests/security/test_embargo_workflow_v1.py tests/security/test_advisory_schema_v1.py tests/security/test_vuln_response_gate_v1.py -v --junitxml=$(OUT)/pytest-vuln-response-v1.xml

test-observability-v2:
	$(PYTHON) tools/collect_trace_bundle_v2.py --seed 20260309 --window-seconds 300 --out $(OUT)/trace-bundle-v2.json
	$(PYTHON) tools/collect_diagnostic_snapshot_v2.py --seed 20260309 --trace-bundle $(OUT)/trace-bundle-v2.json --out $(OUT)/diagnostic-snapshot-v2.json
	$(PYTHON) tools/collect_crash_dump_v1.py --out $(OUT)/crash-dump-v1.json
	$(PYTHON) tools/symbolize_crash_dump_v1.py --dump $(OUT)/crash-dump-v1.json --out $(OUT)/crash-dump-symbolized-v1.json
	$(PYTHON) -m pytest tests/runtime/test_observability_docs_v2.py tests/runtime/test_trace_bundle_v2.py tests/runtime/test_diag_snapshot_v2.py tests/runtime/test_observability_gate_v2.py tests/runtime/test_crash_dump_docs_v1.py tests/runtime/test_crash_dump_capture_v1.py tests/runtime/test_crash_dump_symbolization_v1.py tests/runtime/test_crash_dump_gate_v1.py -v --junitxml=$(OUT)/pytest-observability-v2.xml

test-crash-dump-v1:
	$(PYTHON) tools/collect_crash_dump_v1.py --out $(OUT)/crash-dump-v1.json
	$(PYTHON) tools/symbolize_crash_dump_v1.py --dump $(OUT)/crash-dump-v1.json --out $(OUT)/crash-dump-symbolized-v1.json
	$(PYTHON) -m pytest tests/runtime/test_crash_dump_docs_v1.py tests/runtime/test_crash_dump_capture_v1.py tests/runtime/test_crash_dump_symbolization_v1.py tests/runtime/test_crash_dump_gate_v1.py -v --junitxml=$(OUT)/pytest-crash-dump-v1.xml

test-ops-ux-v3:
	$(PYTHON) tools/run_upgrade_drill_v3.py --seed 20260309 --out $(OUT)/upgrade-drill-v3.json
	$(PYTHON) tools/run_recovery_drill_v3.py --seed 20260309 --out $(OUT)/recovery-drill-v3.json
	$(PYTHON) -m pytest tests/build/test_installer_ux_v3.py tests/build/test_upgrade_recovery_v3.py tests/build/test_rollback_safety_v3.py tests/build/test_ops_ux_gate_v3.py -v --junitxml=$(OUT)/pytest-ops-ux-v3.xml

test-release-lifecycle-v2:
	$(PYTHON) tools/release_branch_audit_v2.py --out $(OUT)/release-branch-audit-v2.json
	$(PYTHON) tools/support_window_audit_v1.py --out $(OUT)/support-window-audit-v1.json
	$(MAKE) test-supply-chain-revalidation-v1
	$(PYTHON) -m pytest tests/build/test_release_policy_v2_docs.py tests/build/test_release_branch_policy_v2.py tests/build/test_support_window_policy_v1.py tests/build/test_release_lifecycle_gate_v2.py -v --junitxml=$(OUT)/pytest-release-lifecycle-v2.xml

test-supply-chain-revalidation-v1:
	$(PYTHON) tools/generate_sbom_v1.py --out $(OUT)/sbom-v1.spdx.json
	$(PYTHON) tools/generate_provenance_v1.py --out $(OUT)/provenance-v1.json
	$(PYTHON) tools/verify_sbom_provenance_v2.py --sbom $(OUT)/sbom-v1.spdx.json --provenance $(OUT)/provenance-v1.json --out $(OUT)/supply-chain-revalidation-v1.json
	$(PYTHON) tools/verify_release_attestations_v1.py --out $(OUT)/release-attestation-verification-v1.json
	$(PYTHON) -m pytest tests/build/test_supply_chain_revalidation_docs_v1.py tests/build/test_sbom_revalidation_v1.py tests/build/test_provenance_verification_v1.py tests/build/test_attestation_drift_v1.py tests/build/test_supply_chain_revalidation_gate_v1.py -v --junitxml=$(OUT)/pytest-supply-chain-revalidation-v1.xml

test-conformance-v1:
	$(PYTHON) tools/run_conformance_suite_v1.py --seed 20260309 --out $(OUT)/conformance-v1.json
	$(PYTHON) -m pytest tests/runtime/test_profile_conformance_docs_v1.py tests/runtime/test_server_profile_v1.py tests/runtime/test_dev_profile_v1.py tests/runtime/test_conformance_gate_v1.py -v --junitxml=$(OUT)/pytest-conformance-v1.xml

test-fleet-ops-v1:
	$(PYTHON) tools/run_fleet_update_sim_v1.py --seed 20260309 --out $(OUT)/fleet-update-sim-v1.json
	$(PYTHON) tools/run_fleet_health_sim_v1.py --seed 20260309 --out $(OUT)/fleet-health-sim-v1.json
	$(MAKE) test-fleet-rollout-safety-v1
	$(PYTHON) -m pytest tests/pkg/test_fleet_policy_docs_v1.py tests/pkg/test_fleet_update_sim_v1.py tests/runtime/test_fleet_health_sim_v1.py tests/runtime/test_fleet_ops_gate_v1.py -v --junitxml=$(OUT)/pytest-fleet-ops-v1.xml

test-fleet-rollout-safety-v1:
	$(PYTHON) tools/run_canary_rollout_sim_v1.py --seed 20260309 --out $(OUT)/canary-rollout-sim-v1.json
	$(PYTHON) tools/run_rollout_abort_drill_v1.py --out $(OUT)/rollout-abort-drill-v1.json
	$(PYTHON) -m pytest tests/pkg/test_rollout_policy_docs_v1.py tests/pkg/test_canary_rollout_sim_v1.py tests/runtime/test_rollout_abort_policy_v1.py tests/runtime/test_fleet_rollout_safety_gate_v1.py -v --junitxml=$(OUT)/pytest-fleet-rollout-safety-v1.xml

test-maturity-qual-v1:
	$(PYTHON) tools/run_maturity_qualification_v1.py --seed 20260309 --out $(OUT)/maturity-qualification-v1.json
	$(PYTHON) -m pytest tests/build/test_maturity_docs_v1.py tests/build/test_maturity_qualification_v1.py tests/build/test_lts_policy_v1.py tests/build/test_maturity_security_response_drill_v1.py tests/build/test_maturity_supply_chain_continuity_v1.py tests/build/test_maturity_rollout_safety_v1.py tests/build/test_maturity_gate_v1.py -v --junitxml=$(OUT)/pytest-maturity-qual-v1.xml

test-desktop-stack-v1:
	$(PYTHON) tools/run_desktop_smoke_v1.py --out $(OUT)/desktop-smoke-v1.json
	$(MAKE) test-gui-app-compat-v1
	$(PYTHON) -m pytest tests/desktop/test_desktop_docs_v1.py tests/desktop/test_display_session_v1.py tests/desktop/test_input_baseline_v1.py tests/desktop/test_window_lifecycle_v1.py tests/desktop/test_desktop_gate_v1.py -v --junitxml=$(OUT)/pytest-desktop-stack-v1.xml

test-gui-app-compat-v1:
	$(PYTHON) tools/run_gui_app_matrix_v1.py --out $(OUT)/gui-app-matrix-v1.json
	$(PYTHON) -m pytest tests/desktop/test_gui_app_compat_v1.py tests/desktop/test_gui_app_compat_gate_v1.py -v --junitxml=$(OUT)/pytest-gui-app-compat-v1.xml

test-compat-surface-v1:
	$(PYTHON) tools/run_compat_surface_campaign_v1.py --out $(OUT)/compat-surface-v1.json
	$(MAKE) test-posix-gap-closure-v1
	$(PYTHON) -m pytest tests/compat/test_compat_docs_v4.py tests/compat/test_posix_gap_closure_v1.py tests/compat/test_process_model_v3.py tests/compat/test_socket_family_expansion_v1.py tests/compat/test_deferred_surface_behavior_v1.py tests/compat/test_compat_surface_gate_v1.py -v --junitxml=$(OUT)/pytest-compat-surface-v1.xml

test-posix-gap-closure-v1:
	$(PYTHON) tools/run_posix_gap_report_v1.py --out $(OUT)/posix-gap-report-v1.json
	$(PYTHON) -m pytest tests/compat/test_posix_gap_closure_v1.py tests/compat/test_deferred_surface_behavior_v1.py tests/compat/test_posix_gap_closure_gate_v1.py -v --junitxml=$(OUT)/pytest-posix-gap-closure-v1.xml

test-hw-matrix-v4: image-blk image-blk-badlen image-blk-badptr image-net
	$(PYTHON) tools/run_hw_matrix_v4.py --out $(OUT)/hw-matrix-v4.json
	$(MAKE) test-hw-baremetal-promotion-v1
	$(PYTHON) -m pytest tests/hw/test_hw_matrix_docs_v4.py tests/hw/test_hw_matrix_v4.py tests/hw/test_driver_lifecycle_v4.py tests/hw/test_baremetal_promotion_v1.py tests/hw/test_hw_negative_paths_v4.py tests/hw/test_hw_gate_v4.py -v --junitxml=$(OUT)/pytest-hw-matrix-v4.xml

test-hw-baremetal-promotion-v1:
	$(PYTHON) tools/collect_hw_promotion_evidence_v1.py --out $(OUT)/hw-promotion-v1.json
	$(PYTHON) -m pytest tests/hw/test_baremetal_promotion_v1.py tests/hw/test_hw_baremetal_promotion_gate_v1.py -v --junitxml=$(OUT)/pytest-hw-baremetal-promotion-v1.xml

test-storage-platform-v1:
	$(PYTHON) tools/run_storage_feature_campaign_v1.py --out $(OUT)/storage-feature-v1.json
	$(MAKE) test-storage-feature-contract-v1
	$(PYTHON) -m pytest tests/storage/test_storage_feature_docs_v1.py tests/storage/test_snapshot_semantics_v1.py tests/storage/test_online_resize_v1.py tests/storage/test_advanced_fs_ops_v1.py tests/runtime/test_platform_feature_profile_v1.py tests/storage/test_storage_platform_gate_v1.py -v --junitxml=$(OUT)/pytest-storage-platform-v1.xml

test-storage-feature-contract-v1:
	$(PYTHON) tools/run_platform_feature_conformance_v1.py --out $(OUT)/platform-feature-v1.json
	$(PYTHON) -m pytest tests/storage/test_storage_feature_docs_v1.py tests/runtime/test_platform_feature_profile_v1.py tests/storage/test_storage_feature_contract_gate_v1.py -v --junitxml=$(OUT)/pytest-storage-feature-contract-v1.xml

test-ecosystem-scale-v1:
	$(PYTHON) tools/run_app_catalog_sim_v1.py --out $(OUT)/app-catalog-sim-v1.json
	$(MAKE) test-app-catalog-health-v1
	$(PYTHON) -m pytest tests/pkg/test_ecosystem_scale_docs_v1.py tests/pkg/test_app_catalog_sim_v1.py tests/pkg/test_pkg_install_success_rate_v1.py tests/pkg/test_catalog_reproducibility_v1.py tests/pkg/test_distribution_workflow_v1.py tests/pkg/test_ecosystem_scale_gate_v1.py -v --junitxml=$(OUT)/pytest-ecosystem-scale-v1.xml

test-app-catalog-health-v1:
	$(PYTHON) tools/run_pkg_install_success_campaign_v1.py --out $(OUT)/pkg-install-success-v1.json
	$(PYTHON) tools/run_reproducible_catalog_audit_v1.py --out $(OUT)/catalog-audit-v1.json
	$(PYTHON) -m pytest tests/pkg/test_ecosystem_scale_docs_v1.py tests/pkg/test_pkg_install_success_rate_v1.py tests/pkg/test_catalog_reproducibility_v1.py tests/pkg/test_distribution_workflow_v1.py tests/pkg/test_app_catalog_health_gate_v1.py -v --junitxml=$(OUT)/pytest-app-catalog-health-v1.xml

test-evidence-integrity-v1:
	$(PYTHON) tools/collect_runtime_evidence_v1.py --out $(OUT)/runtime-evidence-v1.json
	$(PYTHON) tools/audit_gate_evidence_v1.py --evidence $(OUT)/runtime-evidence-v1.json --out $(OUT)/gate-evidence-audit-v1.json
	$(PYTHON) -m pytest tests/runtime/test_evidence_integrity_docs_v1.py tests/runtime/test_runtime_evidence_collection_v1.py tests/runtime/test_gate_evidence_audit_v1.py tests/runtime/test_evidence_trace_linkage_v1.py tests/runtime/test_synthetic_evidence_ban_v1.py tests/runtime/test_evidence_integrity_gate_v1.py -v --junitxml=$(OUT)/pytest-evidence-integrity-v1.xml

test-synthetic-evidence-ban-v1:
	$(PYTHON) tools/collect_runtime_evidence_v1.py --out $(OUT)/runtime-evidence-v1.json
	$(PYTHON) tools/audit_gate_evidence_v1.py --evidence $(OUT)/runtime-evidence-v1.json --out $(OUT)/gate-evidence-audit-v1.json
	$(PYTHON) -m pytest tests/runtime/test_evidence_integrity_docs_v1.py tests/runtime/test_gate_evidence_audit_v1.py tests/runtime/test_synthetic_evidence_ban_v1.py -v --junitxml=$(OUT)/pytest-synthetic-evidence-ban-v1.xml

test-process-readiness-parity-v1:
	$(PYTHON) tools/run_compat_surface_campaign_v2.py --out $(OUT)/compat-surface-v2.json
	$(MAKE) test-posix-gap-closure-v2
	$(PYTHON) -m pytest tests/compat/test_compat_docs_v5.py tests/compat/test_fork_clone_surface_v1.py tests/compat/test_epoll_surface_v1.py tests/compat/test_process_model_v4.py tests/compat/test_deferred_surface_behavior_v2.py tests/compat/test_process_readiness_gate_v1.py -v --junitxml=$(OUT)/pytest-process-readiness-parity-v1.xml

test-posix-gap-closure-v2:
	$(PYTHON) tools/run_posix_gap_report_v2.py --out $(OUT)/posix-gap-report-v2.json
	$(PYTHON) -m pytest tests/compat/test_compat_docs_v5.py tests/compat/test_fork_clone_surface_v1.py tests/compat/test_epoll_surface_v1.py tests/compat/test_deferred_surface_behavior_v2.py tests/compat/test_posix_gap_closure_gate_v2.py -v --junitxml=$(OUT)/pytest-posix-gap-closure-v2.xml

test-isolation-baseline-v1:
	$(PYTHON) tools/run_isolation_campaign_v1.py --out $(OUT)/isolation-campaign-v1.json
	$(MAKE) test-namespace-cgroup-v1
	$(PYTHON) -m pytest tests/security/test_isolation_docs_v1.py tests/security/test_namespace_baseline_v1.py tests/security/test_cgroup_baseline_v1.py tests/security/test_isolation_escape_negative_v1.py tests/security/test_isolation_gate_v1.py -v --junitxml=$(OUT)/pytest-isolation-baseline-v1.xml

test-namespace-cgroup-v1:
	$(PYTHON) tools/run_resource_control_campaign_v1.py --out $(OUT)/resource-control-v1.json
	$(PYTHON) -m pytest tests/security/test_isolation_docs_v1.py tests/security/test_namespace_baseline_v1.py tests/security/test_cgroup_baseline_v1.py tests/security/test_isolation_escape_negative_v1.py tests/runtime/test_resource_control_policy_v1.py tests/security/test_namespace_cgroup_gate_v1.py -v --junitxml=$(OUT)/pytest-namespace-cgroup-v1.xml

test-hw-firmware-smp-v1:
	$(PYTHON) tools/run_hw_matrix_v5.py --out $(OUT)/hw-matrix-v5.json
	$(MAKE) test-native-driver-matrix-v1
	$(PYTHON) -m pytest tests/hw/test_hw_matrix_docs_v5.py tests/hw/test_native_storage_driver_matrix_v1.py tests/hw/test_native_nic_driver_matrix_v1.py tests/hw/test_firmware_table_validation_v3.py tests/hw/test_smp_interrupt_baseline_v1.py tests/hw/test_hw_firmware_smp_gate_v1.py -v --junitxml=$(OUT)/pytest-hw-firmware-smp-v1.xml

test-native-driver-matrix-v1:
	$(PYTHON) tools/collect_firmware_smp_evidence_v1.py --out $(OUT)/hw-firmware-smp-v1.json
	$(PYTHON) -m pytest tests/hw/test_hw_matrix_docs_v5.py tests/hw/test_native_storage_driver_matrix_v1.py tests/hw/test_native_nic_driver_matrix_v1.py tests/hw/test_firmware_table_validation_v3.py tests/hw/test_smp_interrupt_baseline_v1.py tests/hw/test_native_driver_matrix_gate_v1.py -v --junitxml=$(OUT)/pytest-native-driver-matrix-v1.xml

test-hw-matrix-v6:
	$(PYTHON) tools/run_hw_matrix_v6.py --out $(OUT)/hw-matrix-v6.json
	$(SUBMAKE) test-virtio-platform-v1
	$(PYTHON) -m pytest tests/hw/test_hw_matrix_docs_v6.py tests/hw/test_virtio_modern_storage_v1.py tests/hw/test_virtio_modern_net_v1.py tests/hw/test_virtio_scsi_v1.py tests/hw/test_virtio_gpu_framebuffer_v1.py tests/hw/test_driver_lifecycle_v6.py tests/hw/test_hw_negative_paths_v5.py tests/desktop/test_display_device_bridge_v1.py tests/hw/test_hw_gate_v6.py -v --junitxml=$(OUT)/pytest-hw-matrix-v6.xml

test-virtio-platform-v1:
	$(PYTHON) tools/run_hw_matrix_v6.py --out $(OUT)/hw-matrix-v6.json
	$(PYTHON) tools/run_desktop_smoke_v1.py --out $(OUT)/desktop-smoke-v1.json
	$(PYTHON) -m pytest tests/hw/test_hw_matrix_docs_v6.py tests/hw/test_virtio_platform_profile_v1.py tests/desktop/test_display_device_bridge_v1.py tests/hw/test_virtio_platform_gate_v1.py -v --junitxml=$(OUT)/pytest-virtio-platform-v1.xml

test-baremetal-io-baseline-v1:
	$(PYTHON) tools/run_baremetal_io_baseline_v1.py --out $(OUT)/baremetal-io-v1.json
	$(SUBMAKE) test-usb-input-removable-v1
	$(PYTHON) tools/collect_hw_promotion_evidence_v2.py --out $(OUT)/hw-promotion-v2.json
	$(PYTHON) -m pytest tests/hw/test_baremetal_io_profile_v1.py tests/hw/test_usb_input_removable_docs_v1.py tests/hw/test_e1000e_baseline_v1.py tests/hw/test_rtl8169_baseline_v1.py tests/hw/test_xhci_usb_hid_v1.py tests/hw/test_usb_storage_v1.py tests/hw/test_baremetal_io_recovery_v1.py tests/desktop/test_usb_input_focus_delivery_v1.py tests/hw/test_baremetal_io_gate_v1.py -v --junitxml=$(OUT)/pytest-baremetal-io-v1.xml

test-usb-input-removable-v1:
	$(PYTHON) tools/run_baremetal_io_baseline_v1.py --out $(OUT)/baremetal-io-v1.json
	$(PYTHON) tools/run_desktop_smoke_v1.py --input-class usb-hid --input-driver xhci-usb-hid --display-class framebuffer-console --display-driver efifb --boot-transport-class ahci --out $(OUT)/desktop-smoke-v1.json
	$(PYTHON) tools/run_recovery_drill_v3.py --seed 20260309 --out $(OUT)/recovery-drill-v3.json
	$(PYTHON) -m pytest tests/hw/test_usb_input_removable_docs_v1.py tests/hw/test_xhci_usb_hid_v1.py tests/hw/test_usb_storage_v1.py tests/hw/test_baremetal_io_recovery_v1.py tests/desktop/test_usb_input_focus_delivery_v1.py tests/hw/test_usb_input_removable_gate_v1.py -v --junitxml=$(OUT)/pytest-usb-input-removable-v1.xml

test-hw-claim-promotion-v1:
	$(PYTHON) tools/run_hw_claim_promotion_v1.py --out $(OUT)/hw-claim-promotion-v1.json
	$(SUBMAKE) test-hw-support-tier-audit-v1
	$(PYTHON) -m pytest tests/hw/test_support_claim_docs_v1.py tests/hw/test_hw_claim_promotion_v1.py tests/hw/test_hw_support_tier_audit_v1.py tests/hw/test_hw_promotion_regression_v1.py tests/hw/test_hw_support_claim_negative_v1.py tests/hw/test_hw_claim_promotion_gate_v1.py -v --junitxml=$(OUT)/pytest-hw-claim-promotion-v1.xml

test-hw-support-tier-audit-v1:
	$(PYTHON) tools/run_hw_claim_promotion_v1.py --out $(OUT)/hw-claim-promotion-v1.json
	$(PYTHON) tools/run_hw_support_tier_audit_v1.py --out $(OUT)/hw-support-tier-audit-v1.json
	$(PYTHON) -m pytest tests/hw/test_support_claim_docs_v1.py tests/hw/test_hw_claim_promotion_v1.py tests/hw/test_hw_support_tier_audit_v1.py tests/hw/test_hw_support_claim_negative_v1.py tests/hw/test_hw_support_tier_gate_v1.py -v --junitxml=$(OUT)/pytest-hw-support-tier-audit-v1.xml

test-display-runtime-v1:
	$(PYTHON) tools/run_display_runtime_v1.py --out $(OUT)/display-runtime-v1.json
	$(SUBMAKE) test-scanout-path-v1
	$(PYTHON) -m pytest tests/desktop/test_display_runtime_docs_v1.py tests/desktop/test_virtio_gpu_scanout_v1.py tests/desktop/test_efifb_fallback_v1.py tests/desktop/test_display_present_timing_v1.py tests/desktop/test_display_frame_capture_v1.py tests/desktop/test_display_runtime_gate_v1.py -v --junitxml=$(OUT)/pytest-display-runtime-v1.xml

test-scanout-path-v1:
	$(PYTHON) tools/run_display_runtime_v1.py --out $(OUT)/display-runtime-v1.json
	$(PYTHON) tools/capture_display_frame_v1.py --out $(OUT)/display-frame-v1.png
	$(PYTHON) -m pytest tests/desktop/test_display_runtime_docs_v1.py tests/desktop/test_virtio_gpu_scanout_v1.py tests/desktop/test_efifb_fallback_v1.py tests/desktop/test_display_present_timing_v1.py tests/desktop/test_display_frame_capture_v1.py tests/desktop/test_scanout_path_gate_v1.py -v --junitxml=$(OUT)/pytest-scanout-path-v1.xml

test-input-seat-v1:
	$(PYTHON) tools/run_input_seat_runtime_v1.py --out $(OUT)/input-seat-v1.json
	$(SUBMAKE) test-hid-event-path-v1
	$(PYTHON) -m pytest tests/desktop/test_input_seat_docs_v1.py tests/desktop/test_keyboard_event_delivery_v1.py tests/desktop/test_pointer_motion_buttons_v1.py tests/desktop/test_focus_routing_v1.py tests/desktop/test_seat_hotplug_v1.py tests/desktop/test_input_seat_gate_v1.py -v --junitxml=$(OUT)/pytest-input-seat-v1.xml

test-hid-event-path-v1:
	$(PYTHON) tools/run_input_seat_runtime_v1.py --out $(OUT)/input-seat-v1.json
	$(PYTHON) tools/run_hid_event_path_v1.py --out $(OUT)/hid-event-path-v1.json
	$(PYTHON) -m pytest tests/desktop/test_input_seat_docs_v1.py tests/desktop/test_keyboard_event_delivery_v1.py tests/desktop/test_pointer_motion_buttons_v1.py tests/desktop/test_focus_routing_v1.py tests/desktop/test_seat_hotplug_v1.py tests/desktop/test_hid_event_path_gate_v1.py -v --junitxml=$(OUT)/pytest-hid-event-path-v1.xml

test-window-system-v1:
	$(PYTHON) tools/run_window_system_runtime_v1.py --out $(OUT)/window-system-v1.json
	$(SUBMAKE) test-compositor-damage-v1
	$(PYTHON) -m pytest tests/desktop/test_window_system_docs_v1.py tests/desktop/test_surface_lifecycle_v1.py tests/desktop/test_window_zorder_v1.py tests/desktop/test_compositor_damage_regions_v1.py tests/desktop/test_window_resize_move_v1.py tests/desktop/test_window_system_gate_v1.py -v --junitxml=$(OUT)/pytest-window-system-v1.xml

test-compositor-damage-v1:
	$(PYTHON) tools/run_window_system_runtime_v1.py --out $(OUT)/window-system-v1.json
	$(PYTHON) tools/run_compositor_damage_v1.py --out $(OUT)/compositor-damage-v1.json
	$(PYTHON) -m pytest tests/desktop/test_window_system_docs_v1.py tests/desktop/test_surface_lifecycle_v1.py tests/desktop/test_window_zorder_v1.py tests/desktop/test_compositor_damage_regions_v1.py tests/desktop/test_window_resize_move_v1.py tests/desktop/test_compositor_damage_gate_v1.py -v --junitxml=$(OUT)/pytest-compositor-damage-v1.xml

test-real-ecosystem-desktop-v2:
	$(PYTHON) tools/run_real_gui_app_matrix_v2.py --out $(OUT)/real-gui-matrix-v2.json
	$(MAKE) test-real-app-catalog-v2
	$(PYTHON) -m pytest tests/desktop/test_desktop_docs_v2.py tests/pkg/test_ecosystem_scale_docs_v2.py tests/desktop/test_gui_runtime_qualification_v2.py tests/desktop/test_real_desktop_gate_v2.py -v --junitxml=$(OUT)/pytest-real-ecosystem-desktop-v2.xml

test-real-app-catalog-v2:
	$(PYTHON) tools/run_real_pkg_install_campaign_v2.py --out $(OUT)/real-pkg-install-v2.json
	$(PYTHON) tools/run_real_catalog_audit_v2.py --out $(OUT)/real-catalog-audit-v2.json
	$(PYTHON) -m pytest tests/pkg/test_ecosystem_scale_docs_v2.py tests/pkg/test_pkg_install_success_rate_v2.py tests/pkg/test_catalog_reproducibility_v2.py tests/pkg/test_distribution_workflow_v2.py tests/pkg/test_real_catalog_gate_v2.py -v --junitxml=$(OUT)/pytest-real-app-catalog-v2.xml

repro-check:
	@set -e; \
	OUT1="$(OUT)/repro-1"; \
	OUT2="$(OUT)/repro-2"; \
	rm -rf "$$OUT1" "$$OUT2"; \
	$(MAKE) OUT="$$OUT1" build; \
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" OUT="$$OUT1" ISO_NAME=os.iso SOURCE_DATE_EPOCH=1 bash tools/mkimage.sh; \
	$(MAKE) OUT="$$OUT2" build; \
	PATH="$(WSL_PATH)" CC="$(CC)" XORRISO="$(XORRISO)" OUT="$$OUT2" ISO_NAME=os.iso SOURCE_DATE_EPOCH=1 bash tools/mkimage.sh; \
	SHA1=$$(sha256sum "$$OUT1/os.iso" | awk '{print $$1}'); \
	SHA2=$$(sha256sum "$$OUT2/os.iso" | awk '{print $$1}'); \
	echo "repro-check: $$SHA1"; \
	echo "repro-check: $$SHA2"; \
	test "$$SHA1" = "$$SHA2"

clean:
	rm -rf $(OUT)
	cd kernel_rs && $(CARGO) clean 2>/dev/null || true

# --- Legacy C kernel ----------------------------------------------------------

legacy:
	$(MAKE) -C legacy build

# --- Docker (cross-platform) --------------------------------------------------
DOCKER_IMAGE = rugo-builde

docker-all:
	docker build -t $(DOCKER_IMAGE) .
	docker run --rm -v "$(CURDIR):/src" $(DOCKER_IMAGE) bash -c '\
		find /src -maxdepth 5 -type f \( -name Makefile -o -name "*.c" -o -name "*.h" \
		  -o -name "*.asm" -o -name "*.ld" -o -name "*.sh" -o -name "*.py" \
		  -o -name "*.cfg" -o -name "*.conf" -o -name "*.rs" -o -name "*.toml" \) \
		  -exec sed -i "s/\r$$//" {} + \
		&& make clean build image test-qemu'

docker-legacy:
	docker build -t $(DOCKER_IMAGE) .
	docker run --rm -v "$(CURDIR):/src" $(DOCKER_IMAGE) bash -c '\
		find /src -maxdepth 5 -type f \( -name Makefile -o -name "*.c" -o -name "*.h" \
		  -o -name "*.asm" -o -name "*.ld" -o -name "*.sh" -o -name "*.py" \
		  -o -name "*.cfg" -o -name "*.conf" -o -name "*.go" \) \
		  -exec sed -i "s/\r$$//" {} + \
		&& make -C legacy clean build image test-qemu'

