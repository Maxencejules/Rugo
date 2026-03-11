# Build Guide

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| rustup + nightly | latest nightly | Rust compiler + `rust-src` component |
| nasm | any recent | x86-64 assembly |
| ld (binutils) | any recent | Linking kernel ELF |
| xorriso | any recent | ISO image creation |
| cc (gcc/clang) | C99-capable | Build vendored Limine CLI |
| qemu-system-x86_64 | any recent | Smoke tests |
| python3 + pytest | 3.x | Test harness |
| tinygo | 0.40.1 | Go user-space binary (G1 milestone) |
| go | 1.25.3 | Required by TinyGo |

## Building

```bash
make demo-go        # recommended: Rust kernel + Go bootstrap demo
make run-kernel     # kernel-only boot path
make build          # compile kernel ELF
make image          # build kernel-only ISO (no network required)
make image-go       # build TinyGo user-space ISO (os-go.iso)
make image-go-std   # build experimental stock-Go ISO (os-go-std.iso)
make validate       # compatibility alias for make test-qemu
make test-qemu      # full QEMU smoke-test suite
make repro-check    # deterministic ISO gate (build twice + SHA256 compare)
```

Default runtime story:
- `make demo-go` is the recommended front-door demo for the hybrid OS
  It boots `goinit -> gosvcm -> gosh -> timesvc` on the TinyGo lane.
- `make run-kernel` is the kernel-only lane
- `make image-go-std` is experimental and should not be treated as the default
  user-space path

## Windows (PowerShell)

If you are using `mingw32-make` from PowerShell, use:

```powershell
mingw32-make build
mingw32-make image
mingw32-make demo-go
mingw32-make test-qemu
```

The top-level `Makefile` now forces bash recipe execution and defaults to the
GNU Rust toolchain (`nightly-x86_64-pc-windows-gnu`) on Windows to avoid MSVC
linker issues for the kernel ELF.

You still need these tools installed and reachable from bash: `nasm`,
an ISO builder (`xorriso` or `mkisofs`/`genisoimage`), `qemu-system-x86_64`,
and a C compiler (`cc`/`gcc`/`clang`) for Limine CLI.

## Reproducible ISO Check

`make repro-check` performs a deterministic build check by:

1. Building kernel+ISO in `out/repro-1`.
2. Building kernel+ISO again in `out/repro-2`.
3. Forcing reproducible image timestamps via `SOURCE_DATE_EPOCH=1`.
4. Comparing SHA-256 hashes of both ISOs and failing on mismatch.

The ISO creation step uses fixed volume metadata (`-V/-volset/-A/-p/-P`) and,
when `SOURCE_DATE_EPOCH` is set, normalizes mtimes of all files in the ISO
root before calling `xorriso`.

## Pinned External Dependencies

### Limine Bootloader

The Limine bootloader binaries and CLI source are **vendored** in
`vendor/limine/`. No network access is needed during the build.

| Field | Value |
|-------|-------|
| Version | v8.7.0 |
| Branch | `v8.x-binary` |
| Commit | `aad3edd370955449717a334f0289dee10e2c5f01` |
| Date | 2025-01-10 |
| Upstream | https://github.com/limine-bootloader/limine |

**Vendored files:**

| File | Purpose |
|------|---------|
| `limine-bios.sys` | BIOS boot stage (copied into ISO) |
| `limine-bios-cd.bin` | CD-ROM boot stage (copied into ISO) |
| `limine.c` | CLI installer source (compiled at build time) |
| `limine-bios-hdd.h` | Embedded HDD data (included by `limine.c`) |
| `SHA256SUMS` | Integrity checksums |
| `VERSION` | Pinned version metadata |

### How to Update Limine

1. Clone the upstream release branch at the desired tag:
   ```bash
   git clone https://github.com/limine-bootloader/limine.git \
       --branch=v8.x-binary --depth=1 /tmp/limine-update
   ```

2. Copy the required files into the vendor directory:
   ```bash
   cp /tmp/limine-update/limine-bios.sys    vendor/limine/
   cp /tmp/limine-update/limine-bios-cd.bin vendor/limine/
   cp /tmp/limine-update/limine.c           vendor/limine/
   cp /tmp/limine-update/limine-bios-hdd.h  vendor/limine/
   ```

3. Regenerate checksums:
   ```bash
   cd vendor/limine
   sha256sum limine-bios.sys limine-bios-cd.bin limine.c limine-bios-hdd.h > SHA256SUMS
   ```

4. Update `vendor/limine/VERSION` with the new commit hash and version.

5. Run `make clean image test-qemu` to verify the new version works.

6. Commit all changes together.

### Rust Toolchain

Pinned via `rust-toolchain.toml` (nightly channel with `rust-src`).
Rustup manages installation automatically.

### Go / TinyGo version pins

| Tool | Version | Pinned in |
|------|---------|-----------|
| Go | 1.25.3 | `.github/workflows/ci.yml`, `Dockerfile` |
| TinyGo | 0.40.1 | `.github/workflows/ci.yml`, `Dockerfile` |

These are host toolchain installs used in CI and Docker, and also required locally for `make image-go`, `make image-go-std`, and the full `make test-qemu` target.
If you only run `make build` or `make image`, Go/TinyGo are not required.
Update the version numbers in CI and Dockerfile together when upgrading.

