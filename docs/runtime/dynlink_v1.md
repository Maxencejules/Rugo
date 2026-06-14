# Dynamic loading (dlopen/dlsym) — contract v1

Status: boot-verified via `make test-dynlink-v1` (go C4 runtime lane)
Source: `kernel_rs/src/lib.rs` (`sys_dlctl`, `DL_MODULE`, `DLOPEN_BASE`),
dispatch in `kernel_rs/src/syscall.rs` (id 60); `apps/coreutils/dlprobe.asm`.
ABI: `sys_dlctl` (id 60).
Proof: `tests/runtime/test_dynlink_v1.py`.

Full-OS implementation guide Part V.11 (userspace), dynamic loading: load
separately-authored code at runtime, resolve a symbol, and call it
(`dlopen`/`dlsym` semantics).

## Why a module loader, not an ELF `.so` linker (yet)

A real ELF `.so` dynamic linker (dynamic relocations, GOT/PLT) needs the C
toolchain to *produce* `.so` files. That path is **blocked**: mingw's
refptr/auto-import + the homemade `tools/pe_to_elf_v1.py` break C binaries that
cross two pages (proved earlier by `page3probe`: the kernel handles multi-page
apps; the toolchain does not). So v1 demonstrates the **loading mechanism** with a
position-independent module the kernel ships embedded, instead of waiting on the
toolchain.

## Behaviour

A module is `[u32 n_exports][exports: {name[12], u32 offset}][PIC code]`. v1 ships
one module, `dlmod`, exporting `addone` (`lea rax,[rdi+1]; ret` — fully
position-independent, no relocations).

`sys_dlctl(op, name_ptr)`:

- **op 1 = dlopen(name):** for the known module, map a user page at
  `DLOPEN_BASE` **RW**, copy the module image in, then **mprotect it R-X**
  (W^X — never writable and executable at once), and return the base VA.
  Idempotent: a repeat `dlopen` re-loads and returns the same base. `DLOPEN_BASE`
  sits in the free `[0x0180_0000, 0x0200_0000)` gap above the exec-app window, so
  the module never aliases the caller's own ELF segments, heap, or mmap region.
- **op 2 = dlsym(name):** read the **loaded image's** export table from
  `DLOPEN_BASE`, find the symbol, and return `DLOPEN_BASE + offset`.

`dlprobe` then `call`s the resolved address with arg 41 and checks it returns 42
— i.e. the loaded code actually executed in ring 3.

## v1 boundary / carry-forward

- **One embedded PIC module, one load slot.** No on-disk/package `.so`
  files, no ELF parsing, no dynamic relocations (R_X86_64_*), no GOT/PLT, no
  `dlclose`, and a single fixed load address. Those become a real ELF dynamic
  linker once the C `.so` toolchain is fixed (or the C apps move to a real ELF
  linker) — the documented blocker.
- The loaded module is position-independent by construction; a relocating loader
  is what lifts that restriction.

## Acceptance

`make test-dynlink-v1`: the go lane runs `probe dlprobe`; the transcript shows
`DLPROBE: dlsym ok` (dlopen loaded the module, dlsym resolved `addone`, and the
app called it and got 42), with no `DLPROBE: FAIL` and no `USERPF` (the loaded
code executed in ring 3 without faulting), then reaches
`GOINIT: result shutdown-clean` and `RUGO: halt ok`.
