# C apps larger than two pages — contract v1

Status: boot-verified via `make test-bigc-v1` (go lane, app loaded from the
SimpleFS app region).
Source: `apps/c-bigprobe/bigprobe.c`, `tools/pe_to_elf_v1.py`,
`exec_load_app`/`sys_spawn_v1` in `kernel_rs/src/lib.rs`.
Proof: `tests/runtime/test_bigc_v1.py`.

Full-OS implementation guide Part I (toolchain reality) + Part V.11 (rlibc): a C
program that legitimately needs **more than two pages** of image must compile,
link, rewrap to ELF, load, and run correctly. This closes the item that earlier
docs flagged as a PE→ELF "refptr/reloc limit".

## Why there is no real two-page limit

Host `gcc`/`ld` only target PE-COFF, so C apps compile `-mabi=sysv`, link
entirely in PE (no cross-format relocation translation — objcopy mistranslates
PE REL32), and `tools/pe_to_elf_v1.py` re-wraps the fully resolved image as a
one-segment `ET_EXEC` ELF (sections laid at `ImageBase+RVA`, BSS as `memsz`
beyond `filesz`, entry carried over).

The exec window is loaded at **exactly** the PE image base
(`--image-base 0x1400000` == `EXEC_APP_BASE`). Every absolute address baked into
the image — including any `.reloc` base-relocation targets and mingw
`.refptr.<sym>` indirection cells in `.rdata` — is therefore already correct at
load time; no relocation needs to be applied. Image size is irrelevant to this:
a 6-page image loaded at its link base resolves exactly like a 1-page one. The
previously-feared ">2 page" failure was a misdiagnosis; the toolchain path is
size-independent as long as the load base equals the link base.

`pe_to_elf_v1.py` copies `min(rawsize, vsize)` per section into a blob indexed by
RVA (so initialized data on high pages lands at the right VA), zero-fills section
gaps (the blob starts zeroed), and sets `memsz = max(SizeOfImage, max_file_rva)`
so any trailing BSS is zeroed by the loader. The kernel loader
(`exec_load_app`) copies `p_filesz` page-by-page via `as_copyout` and
`as_map_zeroed`s the `p_memsz - p_filesz` BSS tail — the same path the asm
`page3probe` already exercises for three pages.

## The probe (`bigcprobe`)

`apps/c-bigprobe/bigprobe.c` is built like `hello` (crt0 + rlibc, PE link with
`--gc-sections`, then `pe_to_elf_v1.py`) but carries:

- a fully-initialized `const unsigned int bigtab[2048]` (~8 KiB) laid into
  `.rdata`, pushing the image to **~6 pages** (`SizeOfImage` 0x5200, so the
  table's high elements live on the 4th+ page, vaddr ≥ 0x1404000);
- an uninitialized `unsigned int bssbuf[2048]`;
- a `fold()` helper reached only through a `const` function pointer, exercising
  mingw `.refptr` indirection surviving the rewrap.

At runtime it checksums the whole table (a single missing/garbled page changes
the sum), reads `bigtab[1900]` (a high-page element), confirms `bssbuf` reads
back zero across its pages then writes+reads a high slot, and calls `fold`
through the pointer. On full agreement it prints
`BIGC: ok sum=0x14070400 high=0xe1f2a765 pages>2` and `BIGC: done`; any mismatch
prints `BIGC: FAIL …`.

## Acceptance

`make test-bigc-v1`: the test packs a **minimal** app region (just `base-shell` +
`bigcprobe`) onto a fresh disk, boots the go lane, runs `probe bigcprobe`, and the
kernel logs `EXEC: bigcprobe ok` (the >2-page image loaded into a private address
space). The app prints `BIGC: ok sum=0x14070400 high=0xe1f2a765 pages>2` then
`BIGC: done`, reaching `RUGO: halt ok` with no `BIGC: FAIL` and no `USERPF`.

The dedicated region is a **test-disk** accommodation, not a kernel limit: the
shared 40-app region (`tests/conftest.py` `APP_REGION_APPS`) already fills sectors
68..~508 of the 1 MiB test disk, butting against the on-disk VFS at sector 512
(`kernel_rs/src/vfs.rs` `BASE_SECTOR`), so a 41-sector app appended there would
overlap the VFS. The kernel itself caps a single app only at `EXEC_APP_MAX_BYTES`
(64 KiB); a real install sizes the store far above that.

## v1 boundary / carry-forward

- Proves the PE→ELF C-app path is **size-independent** up to `EXEC_APP_MAX_BYTES`
  (64 KiB). The single-`PT_LOAD`, RWX-segment shape (no separate RO/RX/RW
  segments) and load-at-link-base requirement remain; true position-independent
  C apps (PIE, load anywhere) would need real ELF relocation support, which the
  host toolchain still cannot emit cleanly — carry-forward.
