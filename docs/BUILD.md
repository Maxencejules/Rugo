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
| tinygo | 0.40.1 | Go user-space binary (G1 milestone) |
| go | 1.25.3 | Required by TinyGo |
| python3 + pytest | 3.x | Historical validation gates and support tooling |

`python3 + pytest` are not required for the primary developer flow
(`make kernel`, `make userspace`, `make image-demo`, `make boot-demo`,
`make smoke-demo`). They remain required for the full gate suite
(`make gate-all`) and the historical `test-*` targets.

## Building

```bash
make help           # show the primary developer workflows
make kernel         # compile the kernel ELF
make userspace      # build the default TinyGo userspace payload
make build-go-std   # build the supported stock-Go userspace artifacts
make image-demo     # build the default demo ISO (os-go.iso)
make boot-demo      # boot the demo ISO in QEMU
make smoke-demo     # boot + verify serial markers without Python
make image-std      # build the supported stock-Go ISO (os-go-std.iso)
make boot-std       # boot the supported stock-Go ISO in QEMU
make smoke-std      # boot + verify stock-Go serial markers without Python
make image-kernel   # build the kernel-only ISO (os.iso)
make boot-kernel    # boot the kernel-only ISO in QEMU
make gate-all       # full pytest-backed acceptance suite
make repro-check    # deterministic ISO gate (build twice + SHA256 compare)
```

Default runtime story:
- `make image-demo` + `make boot-demo` is the recommended front-door demo
  It boots `goinit -> gosvcm -> gosh -> timesvc` on the TinyGo lane.
- `make smoke-demo` is the fast non-Python serial-marker check for that lane
- `make build-go-std` runs the supported stock-Go bootstrap path and emits:
  `out/gostd.bin`, `out/gostd-contract.env`, and
  `out/runtime-toolchain-contract.env`
- `make image-std` + `make boot-std` is the supported stock-Go boot lane
- `make smoke-std` is the fast non-Python serial-marker check for that lane
- `make image-kernel` + `make boot-kernel` is the kernel-only lane
- `make gate-all` preserves the historical pytest-backed acceptance suite
- the stock-Go lane is supported but non-default; `make image-demo` remains the
  default user-space path

Compatibility aliases remain available: `make build`, `make image`,
`make run-kernel`, `make demo-go`, `make validate`, and `make test-qemu`.

## Windows (PowerShell)

If you are using `mingw32-make` from PowerShell, use:

```powershell
mingw32-make kernel
mingw32-make userspace
mingw32-make image-demo
mingw32-make boot-demo
mingw32-make gate-all
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

These are host toolchain installs used in CI and Docker, and also required
locally for `make userspace`, `make image-demo`, `make build-go-std`,
`make image-std`, and the full `make gate-all` target. If you only run `make kernel` or
`make image-kernel`, Go/TinyGo are not required.
Update the version numbers in CI and Dockerfile together when upgrading.

