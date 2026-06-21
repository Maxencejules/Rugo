# GPT partition table parse — contract v1

Status: boot-verified via `make test-gpt-v1`
Source: `kernel_rs/src/lib.rs` (`gpt_parse_selftest`).
Proof: `tests/runtime/test_gpt_v1.py`.

Full-OS guide Part II.5 (filesystem maturity), partitions: parse a GUID Partition
Table, complementing the MBR parser ([`partitions_v1.md`](partitions_v1.md),
sysinfo op 5).

## Behaviour

At boot `gpt_parse_selftest` reads **LBA 1**, validates the `"EFI PART"`
signature, and reads the header's partition-entry-array LBA (offset 72), entry
count (offset 80), and entry size (offset 84). It then walks the entry array
(`512 / entry_size` entries per sector), counting **live** entries (non-zero
partition type GUID) and logging each one's first/last LBA. When LBA 1 is not a
GPT header (the common case) it reports `GPT: none`.

## Acceptance

`make test-gpt-v1`: on a crafted GPT disk (protective MBR at LBA 0, header at
LBA 1, two entries at LBA 2 — an ESP at LBA 34 and a Linux root at LBA 2082), the
transcript shows `GPT: part first=0x...0022`, `GPT: part first=0x...0822`, and
`GPT: parsed n=0x...0002`, with no `GPT: none` and no `GPT: bad header`.

## v1 boundary / carry-forward

- Read + enumerate only; the v1 self-test inspects the first 16 entries (a larger
  table is valid, just not fully walked).
- **CRC32 of the header and entry array is not validated** (carry-forward), nor
  is the backup GPT at the last LBA consulted.
- Exposing each partition as a block device (offset-shifted reads, like the MBR
  carry-forward) and mounting a filesystem from a GPT partition are carry-forward.
