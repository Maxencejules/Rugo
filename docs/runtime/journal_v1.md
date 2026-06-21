# Filesystem journaling — contract v1

Status: boot-verified via `make test-journal-v1`
Source: `kernel_rs/src/lib.rs` (`sys_sysinfo` op 10),
`apps/coreutils/journalprobe.asm`.
Proof: `tests/runtime/test_journal_v1.py`.

Full-OS implementation guide Part II.5 (filesystem maturity), journaling slice —
write-ahead logging with replay, the crash-consistency primitive the SimpleFS
write-through design lacked.

## Behaviour

`sys_sysinfo` (id 61) **op 10** demonstrates the journal write/replay cycle on
scratch sectors (target 1601, journal header 1602, journal data 1603):

1. **Log (write-ahead):** the intended data is written to the journal *data*
   sector, then a *header* (`"JRNL"` magic, target LBA, `committed=1`) is
   written **last** — the commit point. After this the write is durable in the
   journal even though the target is untouched.
2. **Crash simulated:** the target sector is read and confirmed to still hold
   its pre-write contents (the apply step has not run).
3. **Replay:** the committed header is read; if valid, the journal data is
   copied to the target LBA and the committed flag is cleared (journal
   consumed). This is exactly what mount-time recovery does after a crash
   between commit and apply.
4. The target is re-read and confirmed to hold the journaled data.

Returns 1 and emits `JOURNAL: replay ok` on success.

## v1 boundary / carry-forward

- Single-record journal on fixed scratch sectors, driven by a self-test. Wiring
  the journal in front of the real SimpleFS metadata/data writes (so every
  mutation is logged) and running replay automatically at mount are
  carry-forward, as are a multi-entry ring journal, checksummed records, and
  barriers/FUA ordering guarantees.

## Acceptance

`make test-journal-v1`: `journalprobe` calls op 10; the transcript shows
`JOURNAL: replay ok` and `JOURNALPROBE: ok`, proving log → (crash) → replay →
target updated.
