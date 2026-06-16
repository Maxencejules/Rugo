# rlibc v2 — contract v1

Status: boot-verified via `make test-libc-v1` (the `hello` C program)
Source: `libc/rlibc.c`, `libc/include/rugo/libc.h`; C build flags in `Makefile`
(`RLIBC_CFLAGS` + the `app-hello.elf` link).
Proof: `tests/runtime/test_libc_runtime_v1.py`.

Full-OS guide Part V.11 (userspace libc maturity). rlibc v1 was a thin POSIX-ish
shim (open/read/write, malloc, a printf subset). rlibc v2 adds **errno** and a
fuller **string.h** surface — and, crucially, unblocks growing the library at all.

## The toolchain block, and how v2 gets around it

C apps are compiled to PE-COFF by the host mingw gcc/ld, then rewrapped to ELF by
`tools/pe_to_elf_v1.py`. A prior rlibc-v2 attempt was reverted because adding
functions pushed the `hello` binary from 2 to 3 pages, and the mingw
`refptr`/pseudo-reloc path mistranslates a C app whose image **crosses a second
page** (the OS exec loader itself handles ≥3-page apps correctly — proven by the
pure-asm `page3probe` — so the limit is in the PE→ELF C toolchain, not the kernel).

v2 sidesteps that by enabling **`-ffunction-sections -fdata-sections` +
`--gc-sections`**: each function/datum lands in its own section and the linker
garbage-collects everything not reachable from `_start`. So the library can carry
a full set of helpers while each app links only what it uses. With section GC the
`hello` image actually *shrank* (~7.3 KB → ~5.3 KB; the unused wrappers
mkdir/unlink/stat/pipe/spawn/waitpid/yield are dropped), leaving headroom to add
v2 calls and stay comfortably within two pages (`memsz` 0x1800).

## v2 surface

- **`errno`** (`extern int errno;` + `EIO`): the `open`/`read`/`write` wrappers set
  `errno = EIO` when the kernel ABI returns `RUGO_ERR` (-1).
- **string helpers**: `strcpy`, `strncpy`, `strcat`, `strchr`, `atoi` (joining the
  existing `strlen`/`strcmp`/`strncmp`/`mem*`).

## Acceptance

`make test-libc-v1`: `hello /data/etc/motd` runs from the package store and, after
the v1 markers, prints `HELLOC: errno bad=-1 errno=5` (a failed `open` of a
nonexistent path sets `errno`) and `HELLOC: v2 cpy=rugo chr=go atoi=-123`
(`strcpy`+`strcat` build `"rugo"`, `strchr` finds `"go"`, `atoi("  -123x")` =
-123), then `HELLOC: done`, reaching a clean shutdown with no `USERPF`.

## v2 boundary / carry-forward

- A genuine but **bounded** libc growth: errno + string helpers, made possible by
  section GC. `FILE*` buffered stdio (`fopen`/`fgets`/`fputs`), `snprintf`, and a
  real (non-bump) allocator with `free` remain carry-forward — and a C app that
  legitimately needs **more than two pages** still hits the PE→ELF refptr/reloc
  limit (the underlying toolchain block), so large additions must keep per-app
  reachable size under two pages or wait on a `pe_to_elf` fix.
