# Full-OS Gap Closure Roadmap

> **For agentic workers:** This is the master roadmap for implementing
> `docs/analysis/full-os-gap-analysis.md` §3 (priority-ordered build list).
> Each phase has (or will get) its own detailed plan in this directory.
> Execute phases strictly in order — the gap analysis says foundations block
> everything else.

**Goal:** Close the five dominant absences — dynamic memory management,
dynamic process creation, a real filesystem, TCP/IP, and local input/output —
while honoring Rugo's philosophy.

**Rugo's philosophy (non-negotiable, from README.md + SOURCE_MAP.md):**

1. Every capability claim is backed by **boot-verified runtime code** — a QEMU
   boot with serial marker assertions. Seeded JSON reports are scaffolding,
   never evidence.
2. **Marker discipline**: `PREFIX: message` serial lines, asserted in-order by
   pytest (`_find_in_order`), with count and absence checks.
3. **ABI discipline**: syscall v3 (28 syscalls) is frozen. New syscalls go in a
   documented vNext extension, never by mutating frozen IDs.
4. Hybrid shape: Rust `no_std` kernel (zero external crates), TinyGo-first Go
   userspace. New kernel subsystems are written in-repo, not imported.
5. New core mechanisms land **unconditionally** (no new test-only feature
   lanes) — this incrementally executes gap item 2 (lane unification) instead
   of deepening the lane split.

**Phases (gap-analysis §3 mapping):**

| Phase | Gap item | Plan doc | Status |
|---|---|---|---|
| 1 | §3.1 frame allocator + kernel heap + demand paging | `2026-06-10-mm-foundation.md` | done (`make test-mm-foundation-v1`) |
| 2 | §3.3 preemptive timer scheduling in default lane | `2026-06-10-preemptive-default-lane.md` | done (`make test-sched-preempt-v1`; preemption-safe init protocol, atomic TinyGo allocator, causal test assertions) |
| 3 | §3.2 lift static task limit, dynamic process structures | `2026-06-10-dynamic-tasks.md` | done (`make test-dynamic-tasks-v1`; heap-backed task table, 9 concurrent tasks proven, guard-zoned demand stacks) |
| 4 | §3.4 exec-from-filesystem (spawn+wait) | `2026-06-10-exec-from-filesystem.md` | next |
| 5 | §3.5 VFS + directories over SimpleFS | TBD | pending |
| 6 | §3.6 TCP/IP wired to socket syscalls, DHCP + DNS | TBD | pending |
| 7 | §3.7 keyboard input + framebuffer text console | TBD | pending |
| 8 | §3.8 shell executes external programs; coreutils; pipes | TBD | pending |
| 9 | §3.9 libc-equivalent POSIX-ish layer | TBD | pending |
| 10 | §3.10 parity tier: signals, users/permissions, ASLR + W^X, SMP | TBD | pending |

Each phase ends with: new `tests/**` runtime tests green, a `test-*-v1`
Makefile target, existing gate suites still green, docs updated
(SOURCE_MAP.md + a runtime contract doc), and a commit.
