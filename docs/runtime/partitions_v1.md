# MBR partition table — contract v1

Status: boot-verified via `make test-partitions-v1`
Source: `kernel_rs/src/lib.rs` (`sys_sysinfo` op 5),
`apps/coreutils/partprobe.asm`.
Proof: `tests/runtime/test_partitions_v1.py`.

Full-OS implementation guide Part II.5 (filesystem maturity), partitions
slice — read and parse a classic MBR partition table from the block device.

## Behaviour

`sys_sysinfo` (id 61) **op 5** = MBR parse: reads LBA 0 of the block device
(`block_io_dispatch`), validates the `0x55AA` boot signature at offset 510, and
walks the four 16-byte primary partition entries at offset 446. For each entry
with a non-zero type byte it logs:

```
PART: <index> type=0x<type> lba=0x<start_lba> sectors=0x<sector_count>
```

(all fields are `serial_write_hex`, 16 zero-padded hex digits). It returns the
count of non-empty partitions, or `u64::MAX` if there is no disk or the boot
signature is absent (`PART: no signature`).

## v1 boundary / carry-forward

- **Read-only, primary partitions only.** The four-entry MBR table; no extended
  / logical partitions, no GPT, no partition CHS fields (LBA only).
- **Discovery only.** Partitions are logged, not registered as block ranges or
  mounted — there is no per-partition block device and no filesystem probe yet.
  Mounting a FAT/SimpleFS filesystem found in a partition is carry-forward
  (the natural next slice).
- The boot harness must boot the OS from the CD-ROM (`-boot d`); a data disk
  carrying a valid MBR is otherwise a boot candidate for the firmware.

## Acceptance

`make test-partitions-v1`: the test writes an MBR to LBA 0 of the data disk
with two primary partitions (type `0x83` lba 2048 / 1000 sectors, type `0x0C`
lba 4096 / 2000 sectors). `partprobe` calls op 5; the transcript shows both
`PART:` lines with the exact type/lba/sectors and `PARTPROBE: ok`.
