# Daily-Driver Gap Remediation Plan

Source: the verified 10-point "missing pieces vs a real daily-driver OS" audit
(2026-06-21, 21-agent codebase audit, zero hallucinated evidence). This plan
turns each finding into a concrete remediation: what is already fixed, what is a
contained next step, and what is a milestone that cannot be closed in one pass.

The audit's load-bearing conclusion — **real runnable foundations + a tier of
advanced subsystems that exist only as `cfg(go_test)` boot self-tests or Python
report generators, not live paths** — is correct and frames everything below.

Legend: **[DONE]** landed + gate-verified this pass · **[NEXT]** contained,
days-scale · **[MILESTONE]** weeks+, needs its own roadmap · **[DECISION]**
needs an explicit soften-claim-vs-implement choice before work starts.

---

## 0. What landed this pass (verified)

- **#3 Concurrent pipelines [DONE].** `runPipeline`
  (`services/go/coreutils.go`) now spawns both stages with the non-blocking
  `spawnIO` (mirroring the co-residency `asConc` proves) and reaps both, instead
  of blocking on the left stage via `spawnRunIO`. Left streams into the pipe
  while right drains it in parallel, in separate address spaces; the 512-byte
  ring no longer bounds the left stage's total output. The asm coreutils already
  handled concurrent pipe I/O (`cat.asm` yields-and-retries on a full ring,
  `wc.asm` on an empty one), so no app change was needed. Verified:
  `test-pipes-v1` + `test-concurrent-exec-v1` green.
  - Carry-forward: only two-stage `a | b` is supported (`shell_session.go`
    splits on the first ` | `); 3+ stage pipelines and a `>512 B`
    deadlock-avoidance gate need file-append (`fswrite` overwrites with a 96-byte
    cap today) before they can be expressed through the shell.

- **#8 Salted credentials + iterated KDF + lockout [DONE].**
  `PASSWD`/`login_verify` (`kernel_rs/src/lib.rs`) now:
  - store/check **`PBKDF2-HMAC-SHA256(pw, per-user-salt, 4096)`** (over the
    existing `hmac_sha256`) instead of bare `sha256(pw)` — defeats
    precomputed/rainbow tables and cross-account hash equality, and makes each
    offline guess cost 4096 HMACs;
  - **lock an account after 3 consecutive failures** (`LOGIN_FAILS`/`LOGIN_LOCKOUT`),
    refusing even the correct password until reset — an online brute-force throttle.
  - **file-based store**: credentials now live in a root-owned, owner-only
    `/shadow` VFS file (the `/etc/shadow` analogue; tree rooted at `/data`),
    provisioned at boot (`cred_store_provision`, alongside `dlclose_selftest`).
    `login_verify` reads it as the runtime source of truth (fallback to the seed
    if unreadable). `loginprobe` proves a uid-100 app is denied reading it while
    root can. (Internal `vfs_*` paths are `/data`-stripped → `/shadow`; the login
    syscall path can't reliably create VFS files, so it provisions at boot.)
  - Verified: `test-login-v1` (`lockout ok` + `shadow protected ok`) +
    `test-audit-v1` + `test-users-runtime-v1` + `test-userid-v1` +
    `test-vfs-runtime-v1` (node count 2→3) green.
  - Remaining for #8: a real input fuzzer (see §8). The store is now a file, so a
    `passwd`-style tool to mutate it is a natural follow-up.

- **#4 libc real `free()` + buffered `FILE*` stdio [DONE].** `rlibc.c`:
  - `malloc`/`free` replaced the v1 no-op `free` with a free-list allocator (each
    block carries a 16-byte size header, `free` pushes onto a LIFO list, the next
    fitting `malloc` reuses the freed block). Proof: `HELLOC: heap reuse=1 distinct=1`.
  - Added a **bidirectional buffered `FILE*`** layer over the fd syscalls
    (256-byte buffer, 8-slot pool): read side `fopen`/`fgetc`/`fread`/`feof`,
    write side `fputc`/`fwrite`/`fflush` (flush-on-close), `fclose`. Proofs:
    `HELLOC: stdio fread[16]=from-c-with-love eof=1` and
    `HELLOC: stdio rw[11]=hello-stdio` (write a file, read it back).
  - Verified: `test-libc-v1` (all proofs).
  - Remaining for #4: TLS, a real editor + package installer.

- **#4 distinct errno [DONE].** The syscall ABI returned a single −1 sentinel, so
  rlibc collapsed every failure to `EIO`. Added a per-task `last_errno` (R4Task
  field) that well-defined failure paths stamp with a POSIX code before returning
  −1 (`open` → `ENOENT`/`EACCES`, `read`/`write` → `EBADF`/`EACCES`); new
  `sys_errno` (id 62) returns it — additive, read-only, never changing an existing
  syscall's return convention. `r4_set_errno` is a cfg pair (real in the go lane
  where the task table exists, a no-op elsewhere) so the shared read/write
  handlers compile in every lane. rlibc's `open`/`read`/`write` wrappers now read
  it (`rugo_errno`) and set `errno` to the real cause, falling back to `EIO` only
  on un-stamped paths (so `hello.c` and `test_libc` are unchanged; `hello.elf`
  stays 6016 B — no shared-region pressure).
  - Proof: `errnoprobe` (raw kernel: `ENOENT=2` vs `EBADF=9`, `ERRNO: distinct
    ok`, `test_errno_v1.py`) + `bigprobe` (libc wrappers: `BIGC: errno enoent=1
    ebadf=1 distinct=1`, `test_bigc_v1.py`), both on dedicated disks. Verified +
    all ABI/contract gates green. Coverage now spans `open` (generic-unmatched +
    `/data` + `/tmp` → ENOENT/EINVAL/EACCES) and `read`/`write`/`close`
    (EBADF/EACCES) — `hello.c`'s `open("/no/such/file")` reports the real ENOENT=2
    (`test_libc` upgraded from the old EIO=5). Remaining: stamp `mkdir`/`unlink`/
    `stat`/`lseek`/`spawn` (still `EIO`); negative-return-convention is
    intentionally NOT adopted (it would break the frozen −1 sentinel).

- **#4 dlopen HANDLE TABLE [DONE].** The dynamic linker tracked only ONE live
  object (a single `DL_LOAD_BASE`/`DL_MAP_*`): a second `dlopen` reclaimed the
  first, and `dlsym` always resolved the most-recent object — not POSIX. Added a
  4-entry handle table (`DL_HANDLES{base,lo,hi,ondisk}`): up to 4 objects open
  **concurrently**, each at its own non-overlapping ASLR slot (`dl_aslr_base`
  skips every live handle's slot), each freed only by its own `dlclose`. New
  `sys_dlctl op4 = dlsym_h(handle, name)` resolves a symbol against a *specific*
  handle at that object's base; `op3 dlclose(handle)` releases exactly one. `op1`
  (returns a handle) and `op2` (most-recent dlsym) stay back-compatible, so
  `dlprobe`/`ondlprobe` are untouched. `dl_resolve` now takes an explicit base,
  `dl_load_elf` an `ondisk` flag + registers the handle (unmaps + fails if full).
  - Proof: new ring-3 `multidlprobe` (own dedicated disk) opens `libdl` twice →
    two live objects at distinct bases; `dlsym_h(getval)` on each yields two
    distinct callable VAs (each relocated → 42); `dlclose(h1)` leaves `h2`
    resolvable+runnable while `h1`'s handle returns −1 (`MULTIDL: handle table
    ok`, `test_dlhandles_v1.py`). Verified: 7 dl tests (incl. dynlink/aslr/
    code_aslr/dlopen_ondisk no-regression + boot `DLCLOSE` selftest) + all 18
    ABI/contract gates green. v1 boundary: ≤1 *on-disk* object retained (the
    `DL_LOADED` buffer is shared); embedded handles back onto the static image.

---

## 1. SMP — live per-CPU scheduler  [MILESTONE — highest leverage]

Current (`kernel_rs/src/smp.rs`, `lib.rs`): cross-CPU data locks
(`FS<NET<STORAGE<R4_RQ`, `smp.rs:1869`) are real and wired into production
syscall/VFS/net paths, but **no task runs on an AP at steady state**:
`ap_eligible` is forced false by the production initializer (`lib.rs:4068`) and
flipped true only inside `cfg(go_test)` self-tests; `SMP_LIVE_MODE` is set to 1
transiently in three self-tests and reset to 0, so `ap_pull_r4_task` is inert
after boot. `R4_CURRENT` is a single global (`lib.rs:4004`). No load balancer,
work-stealing, or migration exists.

Remediation (slices, each its own `test-*-v1` boot gate):
1. **Per-CPU current task.** Replace the single `R4_CURRENT` with a per-CPU
   array indexed by `r4_current_smp()`'s CPU id (the GS-base machinery already
   exists). Make `r4_switch_to`/timer preemption per-CPU. Gate: BSP-only still
   green (no behavior change yet).
2. **Live AP run path.** Promote `SMP_LIVE_MODE` to a real, default-on flag and
   let APs pull `Ready` tasks from a shared queue under `R4_RQ_LOCK` instead of
   parking. Start with one AP, spawned-app tasks only. Gate: a spawned app
   observably runs on CPU≠0 (read APIC id from the task).
3. **Load balancing + migration.** BSP pushes excess `Ready` tasks to idle APs;
   add work-stealing when an AP's queue is empty. Gate: N>cores spawns spread
   across all CPUs; per-CPU dispatch counters all non-zero.
4. **SMP-safe audit.** Re-audit every syscall handler for per-CPU `current`
   assumptions now that two tasks run truly concurrently (the locks exist but
   have never contended against a live AP task).

Risk: high — concurrency bugs are non-deterministic and the `-smp1` lanes can't
reproduce them. Each slice must land behind a multi-core boot gate.

**Investigated this pass (2026-06-21) — why this is the capstone, not a slice.**
The live-scheduler *mechanism* fully works: `smp_live_sched_selftest`
([smp.rs:2199](../../kernel_rs/src/smp.rs#L2199)) creates real per-address-space
tasks, marks them `ap_eligible`, flips `SMP_LIVE_MODE=1`, and APs autonomously
pull + run them under `R4_RQ_LOCK` ([smp.rs:2105](../../kernel_rs/src/smp.rs#L2105)).
BUT those tasks run a tiny code blob and retire via a **dedicated `int 0x81` path**
(`ap_user_trap`/`ap_user_done`). A real `sys_spawn` app does general `int 0x80`
syscalls and exits via `r4_exit_and_switch` — and **AP-side task exit / blocking /
rescheduling is the unimplemented part**. `ap_resume_r4_task` sets the AP's
per-CPU current (`gs:[16]`) so an app's syscalls would dispatch via
`r4_current_smp()`, but returning the AP to its park loop on the app's exit/block
is not handled. So the first *real* slice is: implement AP-side exit/block/resched
(generalize `ap_user_done` to the real scheduler), then mark spawned external apps
`ap_eligible` with `SMP_LIVE_MODE` on. `test-smp-v1` today exercises the `int 0x81`
selftest path, **not** a real app's path, so a new spawn-an-app-on-an-AP gate is
also required. This is genuine capstone work, not a flag flip.

---

## 2. POSIX/runtime surface  [epoll DONE; rest MILESTONE]

- **[DONE] Native epoll** (`sys_epoll`, ABI v3.x id 55). A real level-triggered
  readiness set over the existing fd/pipe tables — "stateful poll", reusing the
  `sys_poll_v1` readiness rules. ops: create / ctl_add / wait / close, writing
  `{fd:i32, revents:u16, pad:u16}` records (EPOLLIN=0x1, EPOLLOUT=0x4). Verified
  by a ring-3 `epollprobe` on a dedicated disk (`test-epoll-v1`,
  `EPOLLPROBE: ready ok`): an empty pipe reports 0 ready, after a write reports
  the read end ready with EPOLLIN. The additive id was allocated cleanly — all
  five ABI gates and the Linux-compat epoll-deferral contract stay green (the
  compat profile's deferral is a separate, report-backed surface).

Still missing for #2: fork/clone work via `sys_proc_ctl` (real CoW fork,
`lib.rs:5598`); the direct syscall IDs 43/44/45 are deliberate ENOSYS stubs
(`lib.rs:12518`) **by tested contract** — do not wire them. io_uring / netlink /
raw-packet / namespaces / cgroups have zero kernel implementation; each is an
independent milestone (defer until a userland consumer exists; keep honest
ENOSYS until then).

Remediation (pick by what userland actually needs next):
- **epoll first** — a level-triggered readiness set over the existing fd/pipe
  tables is the smallest real win and unblocks event-loop userland. The kernel
  already computes per-fd readiness in `sys_poll_v1` ([lib.rs:2216](../../kernel_rs/src/lib.rs#L2216)),
  so epoll is essentially *stateful poll* and the readiness logic is reusable.
  - **Investigated this pass (2026-06-21): epoll is an ABI-allocation decision,
    not a casual patch.** Free syscall id 55 exists in the 48..63 window and no
    go-lane *runtime* test asserts epoll=ENOSYS (the deferral is a Linux-compat
    contract asserted only by `tests/compat/` Python report generators). BUT the
    syscall surface is frozen behind **five ABI gates** —
    `test_abi_source_truth_v3`, `test_abi_docs_v3`, `test_abi_window_v3`,
    `test_abi_stability_gate_v3`, `test_abi_diff_gate_v3` — so adding id 55 means
    bumping the frozen ABI baseline + docs in lockstep across all five (plus a new
    probe app in `COREUTILS_ELFS` + `APP_REGION_APPS`). That is a deliberate
    governance step the project gated on purpose; do it as an explicit ABI v3.x
    additive allocation, then implement `sys_epoll` (create/ctl/wait) reusing the
    poll readiness logic, with a ring-3 `epollprobe` runtime test.
- **Do NOT wire the direct fork/clone/epoll IDs (43/44/45)** to the working
  path: they are a *deliberately-tested deferral contract* (`tests/compat/`), not
  a bug. Allocate a fresh id for a real implementation instead.
- namespaces/cgroups/io_uring/netlink are each independent milestones; defer
  until there is a userland consumer, and keep them as honest ENOSYS until then.

---

## 3. Concurrent pipelines — **DONE** (see §0). Follow-ups: 3+ stage pipelines and `fswrite` append.

---

## 4. Userland / libc  [MILESTONE, with contained slices]

Current: the dynamic loader (`dl_load_elf`/`dl_resolve`/`sys_dlctl`, dlsym,
dlclose) is real but `cfg(go_test)`-only and single-global-object (no handle
table, no dependency resolution, `lib.rs:6676`). libc (`libc/rlibc.c`) is thin:
no `FILE*`, only `EIO`, no TLS, no-op `free`. No editor; `pkgsvc.go` is a
verifier, not an installer.

Remediation slices:
- **libc errno table + real `stdio` `FILE*`** over the existing fd syscalls
  (buffered read/write). Contained; gated by a new `rlibc` proof app.
- **`free` that actually frees** (bump → free-list) in the user allocator.
- **dlopen handle table** (multiple simultaneous objects) + needed-library
  resolution — promotes the loader past the single-object model.
- A real line editor and a package *installer* (not just verifier) are larger.

Watch: the Go userspace image is at ~31.8 KiB of the 28 KiB… (32 KiB) cap with
≈0.9 KiB headroom — Go-side growth needs the 9th code page procedure first.

---

## 5. Hardware breadth  [MILESTONE + DECISION on doc honesty]

Current: every native driver (VirtIO-blk/net, NVMe, e1000, xHCI/HID, MSI) is a
feature-gated **polled** proof; none is continuously device-interrupt-driven on
its data path (`native.rs:897` MSI handler only counts + EOIs). No power
mgmt/suspend; only S5 (`lib.rs:7187`).

**[DECISION] Doc-honesty debt:** `docs/hw/support_matrix_v7.md:18-41` declares
**AHCI Tier-1 release-blocking** with *zero* kernel source; rtl8169 and IOMMU are
likewise doc-only. Either:
- (a) **Implement** a minimal AHCI/rtl8169 driver to back the Tier-1 claim, or
- (b) **Downgrade** the matrix to reflect "no implementation" / planned tier.

Note (b) is **not free**: the matrix is asserted by ~15 `tests/hw/*` gates
(`test_hw_gate_v7`, `test_hw_matrix_docs_v6`, `test_nvme_ahci_docs_v1`,
`test_native_storage_driver_matrix_v1`, …). Softening the claim requires updating
those gates in lockstep. This is a deliberate call for the owner — flagged, not
silently changed.

Real remediation: make one driver (NVMe is closest) genuinely interrupt-driven
end-to-end as the template, then widen.

**Investigated this pass — the "polled" claim is narrower than stated.** INPUT is
already genuinely interrupt-driven: the PS/2 keyboard on **IRQ1** (vector 33) and
the mouse on **IRQ12** (`MOUSE: irq dx/dy`, `test_mouse_irq_v1.py`) drive their
data paths off real hardware interrupts, not polling. The MSI/MSI-X plumbing is
also real: `bind_irq`/`enable_msi[x]` program vector 64, and `handle_irq`
([native.rs:897](../../kernel_rs/src/runtime/native.rs#L897)) counts + EOIs each
MSI (`irq_hits=` is reported per NVMe I/O). The precise residual is **storage/net
COMPLETION**: `nvme_submit_command` ([native.rs:704](../../kernel_rs/src/runtime/native.rs#L704))
binds + receives the MSI but detects completion by **spin-polling the CQ phase
bit** (line 753) rather than blocking on the interrupt. Flipping that to a
`hlt`-until-IRQ wait is the genuine fix — but it sits on the **load-bearing**
storage path (the FS reads/writes through it) in the higher-friction native lane,
so it is a real milestone slice (template-then-widen), not a safe one-pass change.

---

## 6. GUI as a standing system  [MILESTONE + DECISION]

Current: framebuffer + pixel/alpha/cursor primitives (`fb.rs`) and IRQ12 mouse
input (`kbd.rs`) are real; the compose/surface/input-poll syscalls are real
kernel code but `cfg(go_test)`-gated with **zero userspace callers**. The booted
"desktop" (`services/go/desktop.go:23-62`) renders nothing — scripted `log()` +
`sysYield()`. The WM/toolkit/GUI-runtime "qualification" is
`tools/x4_desktop_runtime_common_v1.py:612-629` grepping the serial boot
transcript for scripted `DESK*` markers; budgets are asserted, not measured.

**[DECISION] Doc-honesty debt:** the desktop report presents transcript-grep as
measured compositor/app-launch budgets. Either build a real (even minimal)
window-server service that drives the GUI syscalls and emit *measured* numbers,
or relabel the report as a scripted-marker contract check (again gated by
~10 `tests/desktop/*` gates — lockstep update required).

Real remediation: a persistent window-server task that owns surfaces + an event
loop pumping the real input ring, with one userspace client drawing through the
compose syscall. That is the first genuinely "standing" slice.

**[DONE — first standing slice] Persistent owner-stamped surface registry.** The
compositor was a one-shot z-order of a client's *throwaway* per-call list (op 4) —
no persistence, no ownership, no lifecycle. Added a persistent registry
(`WM_SURFACES`, 8 slots) on `sys_ioctl`: **op 8 `wm_register`** (stamps the
caller's tid, persists across calls, no cross-client hijack), **op 9 `wm_compose`**
(z-orders the *whole* registry — every live client's window), **op 10 `wm_clear`**
(owner-checked). `wm_release_owner` runs from `r4_exit_and_switch`, so a dead
client's windows disappear — the server-owned per-client lifecycle. Now driven by
**real userspace callers** (closing the "zero userspace callers" gap): `wmprobe`
registers two windows, composes (=2), clears one, composes (=1), exits leaving one;
a *different* client `wmcheck` composes and sees **0** — the kernel exit-cleaned
the dead owner's window (`test_winsrv_v1.py`). This is the registry + lifecycle a
window server is built on. Still carry-forward: a **resident user-space compositor
process** driving the compose loop (vs clients triggering it), **two concurrently-
live clients** on screen (proven here across two *sequential* clients), shared-
memory pixel surfaces, and input routing to the focused window. The fabricated
`DESK*` transcript-grep DECISION is unchanged (separate doc-honesty debt).

---

## 7. Installer / self-hosting  [MILESTONE]

Current: `installer_selftest` (`lib.rs:11082`) is genuine runtime provisioning
but writes a fill pattern (not the real kernel/fs), installs no bootloader, and
restores the boot disk so nothing boots standalone. `build_installer_v2.py` and
the graphical smoke tool are report generators. Self-hosting is absent.

Remediation (ordered):
1. Installer writes the **real** kernel ELF + SimpleFS image to the target disk.
2. Install the Limine bootloader stages to the target so it boots unaided.
3. Boot the installed target standalone in QEMU as the gate (not a restore).
4. Self-hosting (Rugo building Rugo) is a separate, much larger milestone.

---

## 8. Security hardening depth  [salt DONE; rest MILESTONE + DECISION]

- **[DONE]** Salted credential hashing (§0).
- **[NEXT] Iterated KDF + `/etc/shadow`.** Replace the single-pass
  `sha256(salt||pw)` with an iterated KDF (PBKDF2-HMAC-SHA256 over the existing
  `hmac_sha256`, fixed cost) and move `PASSWD` out of a kernel static into a
  root-owned VFS file. Add login attempt lockout/backoff.
- **[DONE] Public-key package/update signing.** The symmetric keyed-hash
  (`SHA-256(key||payload)`, kernel HMAC) is now joined by a genuine **asymmetric
  Lamport one-time signature** verifier (`kernel_rs/src/pqsig.rs`): the kernel
  embeds ONLY the public key (256 SHA-256 hash pairs, `lamport_pub.bin` 16 KiB) +
  a reference signature, so it can verify but never forge — the property the
  symmetric scheme lacked. `lamport_verify` reuses the in-kernel SHA-256 (no
  big-int/curve math → small, auditable; sidesteps the "constant-time field
  implementation" risk entirely). Keypair+sig generated offline + deterministically
  by `tools/lamport_keygen_v1.py` (private key discarded). `sys_sigverify` (id 63):
  op1 accepts the genuine sig, op2/op3 reject a message/signature tamper. Proven
  at boot (`PQSIG: lamport verify ok, forgery rejected`) AND from ring 3
  (`PQSIGAPP: verify ok forge rejected`, `test_pqsig_v1.py`). Carry-forward
  (`pqsig_v1.md`): Lamport is one-time, so per-package signing needs a Merkle/XMSS
  batch (the 8 KiB sig is large for the size-tight package store) — the verify
  primitive + asymmetry are what's closed here; wiring it into the live
  `sys_spawn` package load (vs the boot/ring-3 proof) is the remaining step.
- **[DECISION] Fabricated fuzzing.** `tools/run_security_fuzz_v2.py:54,65`
  invents `signal_hits` via `rng.random()<0.004` and `coverage_points` via a
  hash — telemetry that *looks* measured but never feeds input to a real binary.
  Either build a real input-mutation harness against an actual parser
  (ELF/GPT/packet), or relabel these as a synthetic control-attestation model so
  they stop reading as measured fuzz coverage. Gated by 5 `tests/security/*`
  gates — lockstep update required.
- Parser hardening: the audit **retracts** the "31 unchecked unwraps" concern —
  all are infallible `try_into().unwrap()` on pre-bounds-checked fixed slices.
  No action; do not cite it as evidence.

---

## 9. Kernel structure / maintainability  [PATTERN DEMONSTRATED; bulk MILESTONE]

Current: `lib.rs` ~13.6k lines / 249 fns holding syscalls + ELF loader + linker +
GPT/FAT16 + serial despite 22 sibling modules. Build emits **91 warnings**
(confirmed), none denied. The unwrap concern is withdrawn (see #8).

- **[DONE] Seven clean module extractions — `cred`, `epoll`, `rng`, `audit`,
  `dmesg`, `gpt`, `fat16`.**
  - `cred.rs` (now 189 lines): the password table + iterated salted KDF + lockout
    + `/shadow` store + `login_verify`, depending only on `crate::sha256` +
    `crate::vfs`; the login dispatch calls `crate::cred::login_verify`.
  - `epoll.rs` (110 lines): the epoll-instance state (`EPOLLS`) + `sys_epoll` op
    dispatch, depending only on `crate::memory::*` and the `epoll_fd_ready`
    readiness helper (which stays in `lib.rs` with the fd/pipe tables it reads —
    so no fd internals are exposed). The dispatch calls `crate::epoll::sys_epoll`.
  - `rng.rs` (125 lines): the xorshift64*/RDRAND CSPRNG + `rng_next` +
    `sys_getrandom` + hwseed self-test, depending only on `crate::memory::copyout_user`
    and the entropy sources (`crate::cmos_unix_seconds`, `crate::R4_PREEMPT_TICKS`,
    `crate::serial_write`) which stay in lib.rs.
  - `audit.rs` (98 lines): the security audit ring + `audit_event` + `audit_read`
    + checkpoint self-test, depending only on `crate::R4_CURRENT`,
    `crate::memory::copyout_user`, and `crate::{slice_contains,serial_write}`.
  - `dmesg.rs` (47 lines): the kernel-log ring (`klog_append`/`klog_read`) that
    `serial_write` mirrors into; depends only on `crate::memory::copyout_user`.
  - `gpt.rs` (136 lines): GPT partition-table parse + IEEE CRC-32 + CRC self-test,
    depending on `crate::storage`, `crate::block_io_dispatch`, `crate::BLK_DATA_PAGE`
    (incl. its tuple field, via descendant access), and `crate::serial_write*`.
  - **Key technique: no `pub(crate)` widening needed** — child modules can read
    *private* crate-root items (the descendant-access privacy rule), so the
    entropy/tid/serial helpers stay private in lib.rs and are consumed via
    `crate::` paths. (One caveat learned: the rule is one-directional — a parent
    can't read a child module's privates, so a self-test that touches a module's
    internal state must live *in* that module.)
  - All four are `go_test`-gated, so the blast radius is the go lane (verifiable
    without the full 50-lane gate). These are the clean **template** for the rest.
  - `fat16.rs` (346 lines): the FAT16 on-disk parser — `fat16_read_named`,
    `fat16_read_chain`, `fat16_write_named`, `fat16_list` — moved verbatim,
    depending on `crate::{block_io_dispatch, storage, BLK_DATA_PAGE, serial_write*}`
    via descendant access. The `/mnt` `FAT_FILE` cache and fd integration stay in
    lib.rs and call through `crate::fat16::` (7 call sites rewritten). This is the
    single largest #9 bulk item the earlier passes had flagged as deferred — it
    proved a *clean* extraction after GPT left the four functions contiguous and
    inter-call-free. Verified: FAT16 read/chain/write + partitions + login/epoll
    sanity all green.
  - **Key technique: no `pub(crate)` widening needed** — child modules can read
    *private* crate-root items (the descendant-access privacy rule), so the
    entropy/tid/serial helpers stay private in lib.rs and are consumed via
    `crate::` paths. (One caveat learned: the rule is one-directional — a parent
    can't read a child module's privates, so a self-test that touches a module's
    internal state must live *in* that module.)
  - Net effect: despite adding TWO whole features this session (epoll + /shadow),
    `lib.rs` is **net-REDUCED 695 lines** — 13,517 (start) → 12,822 — with ~1,050
    lines now in **seven** focused modules. The direction is decisively
    decompose-not-grow. The remaining monolith bulk (the fd-integrated `/mnt`
    cache, the fd/pipe layer, the ELF/PIE loader) is larger, interconnected, and
    spread across the file (many call sites), so it needs the full-gate subsystem
    extraction.

Remaining (the bulk, MILESTONE):
- Extract the remaining self-contained subsystems the same way — GPT/FAT16
  parsing, the ELF/PIE loader, the FILE/pipe/poll/epoll layer, the fb/console.
  Subsystems used across many lanes (un-gated) invalidate all ~50 lane builds, so
  batch those behind one full gate.
- Triage the 91 warnings: delete genuinely-dead code, `#[allow]` the intentional,
  reach a clean build, then `-D warnings` in CI. Most are cross-lane `dead_code`,
  so this also needs the full gate (a fn unused in the go lane may be live in
  another).

Risk: low logic risk, high build/iteration cost — schedule when a full gate run
is affordable.

---

## 10. Evidence vs implementation clarity  [ongoing discipline]

The audit confirms the pattern (e.g. `sys_net_query` ops 1-3 live, 4-25
self-test, `lib.rs:4245`) but also that the docs themselves keep the distinction
visible (`full-os-implementation-status.md`, M84's explicit warning). Keep that
discipline: when a milestone's gate is a boot self-test or a report generator
rather than a live path, say so in the contract doc. The §5/§6/§8 doc-honesty
decisions above are the concrete instances to resolve.

---

## Suggested order

1. **SMP slice 1-2** (§1) — highest leverage; makes the concurrency story real.
2. **libc errno + stdio/`free`** (§4) and **KDF + /etc/shadow** (§8) — contained
   userland/security wins.
3. **Resolve the three doc-honesty decisions** (§5, §6, §8-fuzz) — cheap once the
   soften-vs-implement call is made; each needs lockstep gate updates.
4. **lib.rs extraction + warning cleanup** (§9) — batch behind one full gate.
5. **epoll** (§2), then the larger milestones (drivers §5, window server §6,
   installer §7) as dedicated roadmaps.
