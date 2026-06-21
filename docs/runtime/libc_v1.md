# rlibc Contract v1 (libc-equivalent)

Status: live runtime (boot-verified)
Source: `libc/` (crt0.asm, rlibc.c, include/rugo/libc.h),
`tools/pe_to_elf_v1.py`, proof program `apps/hello-c/hello.c`
Proof: `make test-libc-v1`, `tests/runtime/test_libc_runtime_v1.py`

Closes gap-analysis build-list item 9: a POSIX-ish C layer over the
int 0x80 ABI v3 surface, so software written in C can be compiled
against Rugo and run from the package store.

## Surface (v1)

- process: `exit`, `yield`, `spawn` (name + args + optional pipe
  stdin/stdout), `waitpid`
- files (`/data` tree): `open` (O_RDONLY/O_WRONLY/O_RDWR/O_CREAT),
  `read`, `write`, `close`, `mkdir`, `unlink`, `stat_kind_size`
- pipes: `pipe2fds` (fs_ctl op 4)
- heap: `malloc`/`free`/`sbrk` — bump allocator in the demand-paged
  exec window `[0x0160_0000, 0x017F_0000)`; `free` is a no-op in v1
- strings: `memset`, `memcpy`, `memmove`, `memcmp`, `strlen`, `strcmp`,
  `strncmp`
- stdio: `putchar`, `puts`, `printf` (`%s %c %d %u %x %X %%`),
  single-write line buffering so concurrent tasks cannot splice output;
  output goes to the console or to the process's stdout pipe fd
- entry state: crt0 publishes `rugo_args`/`rugo_args_len` (the spawn
  argument string) and `rugo_stdin_fd`/`rugo_stdout_fd`

## Toolchain (documented host quirk)

The host gcc/binutils only target PE-COFF, and objcopy mistranslates PE
REL32 relocations to ELF (the implicit -4 addend is lost, skewing every
call). So C programs are compiled `-mabi=sysv -ffreestanding` and
**linked entirely in PE** (`ld -m i386pep --image-base 0x1400000`),
then `tools/pe_to_elf_v1.py` rewraps the fully resolved image — no
relocations left — as a one-segment ET_EXEC ELF for the package store.
The app size cap is 64 KiB (`EXEC_APP_MAX_BYTES`).

## Proof program

`apps/hello-c/hello.c` (`hello [path]` in the shell) exercises printf
formatting, malloc, the spawn argument string, and open/read of a file
created moments earlier through the shell — all through rlibc.

## Distinct errno (Part V.11)

`errno` no longer collapses every failure to `EIO`. The kernel stamps a
per-task error code on well-defined failure paths (`open` → `ENOENT`/`EACCES`,
`read`/`write` → `EBADF`/`EACCES`) and `sys_errno` (id 62) returns it; the rlibc
`open`/`read`/`write` wrappers read it (`rugo_errno`) and set `errno` to the
real cause, falling back to `EIO` only where the kernel has not yet stamped a
code. `apps/c-bigprobe/bigprobe.c` proves it on its dedicated disk:
`open("/data/nope")` → `errno == ENOENT`, `read(99)` → `errno == EBADF`, and the
two differ (`BIGC: errno enoent=1 ebadf=1 distinct=1`, `test_bigc_v1.py`). The
raw kernel surface is proven independently by `errnoprobe` (`test_errno_v1.py`).

## v1 carry-forward

A real allocator with free and buffered FILE* streams are **done** (see
`bigprobe`); remaining: TLS, more errno coverage (only the highest-traffic
failure paths are stamped so far — others still fall back to `EIO`), lseek
(needs a kernel seek surface), environment variables, and a port of a real
third-party C program.
