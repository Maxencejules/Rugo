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
       build-fs image-fs \
       build-fs-badmagic image-fs-badmagic \
       build-pkg-hash image-pkg-hash \
       build-net image-net \
       build-go image-go \
       run test-qemu repro-check clean legacy docker-all docker-legacy

# Tools
NASM    ?= nasm
LD      ?= ld
CC      ?= cc
XORRISO ?= xorriso
PYTHON  ?= python3
WSL_PATH ?= /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/mnt/c/mingw64/mingw64/bin

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

run: image
	./tools/run_qemu.sh

test-qemu: image image-panic image-pf image-idt image-sched image-user-hello image-syscall image-syscall-invalid image-stress-syscall image-stress-ipc image-stress-blk image-pressure-shm image-yield image-user-fault image-ipc image-ipc-badptr-send image-ipc-badptr-recv image-svc-badptr image-ipc-buffer-full image-ipc-waiter-busy image-ipc-svc-overwrite image-svc-full image-svc-bad-endpoint image-shm image-quota-endpoints image-quota-shm image-quota-threads image-blk image-blk-badlen image-blk-badptr image-blk-invariants image-fs image-fs-badmagic image-pkg-hash image-net image-go
	$(PYTHON) -m pytest tests/ -v

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

