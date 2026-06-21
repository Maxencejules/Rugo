//! GPT partition-table parsing self-tests (full-os guide Part II.5, partitions).
//!
//! Extracted from `lib.rs` (gap #9, maintainability). Reads the on-disk GPT via
//! the shared block scratch buffer; depends on `crate::storage`,
//! `crate::block_io_dispatch`, `crate::BLK_DATA_PAGE`, and `crate::serial_write*`,
//! all reached via `crate::` paths (descendant access — no `pub(crate)` widening).

use crate::{block_io_dispatch, serial_write, serial_write_hex, storage, BLK_DATA_PAGE};

/// GPT partition-table parse self-test: read the GPT header at LBA 1, validate the
/// "EFI PART" signature, then walk the partition-entry array and count the live
/// entries (non-zero type GUID), logging each one's first/last LBA. Runs at boot;
/// reports `GPT: none` when LBA 1 is not a GPT header (the common case).
pub(crate) unsafe fn gpt_parse_selftest() {
    if !storage::r4_storage_available() || !block_io_dispatch(false, 1, 512, false) {
        serial_write(b"GPT: none\n");
        return;
    }
    if BLK_DATA_PAGE.0[0..8] != *b"EFI PART" {
        serial_write(b"GPT: none\n");
        return;
    }
    let entry_lba = u64::from_le_bytes(BLK_DATA_PAGE.0[72..80].try_into().unwrap());
    let num_entries = u32::from_le_bytes(BLK_DATA_PAGE.0[80..84].try_into().unwrap()) as u64;
    let entry_size = u32::from_le_bytes(BLK_DATA_PAGE.0[84..88].try_into().unwrap()) as u64;
    // GPT entries are >= 128 bytes (UEFI spec); bounding entry_size to [128, 512]
    // also guarantees every per-sector entry's fields (off + 48) stay within the
    // 512-byte BLK_DATA_PAGE, so a malformed header cannot drive a slice-OOB
    // panic at boot.
    if entry_size < 128 || entry_size > 512 || entry_lba == 0 {
        serial_write(b"GPT: bad header\n");
        return;
    }
    let per_sector = 512 / entry_size;
    // v1 bound: inspect the first 16 entries (typically 4 sectors). A larger
    // table is valid; this self-test just confirms the parse, not full coverage.
    let max_entries = core::cmp::min(num_entries, 16);
    let mut count = 0u64;
    let mut e = 0u64;
    'walk: while e < max_entries {
        let sector = entry_lba + e / per_sector;
        if !block_io_dispatch(false, sector, 512, false) {
            break 'walk;
        }
        let mut j = 0u64;
        while j < per_sector && e < max_entries {
            let off = (j * entry_size) as usize;
            let mut used = false;
            let mut k = 0usize;
            while k < 16 {
                if BLK_DATA_PAGE.0[off + k] != 0 {
                    used = true;
                    break;
                }
                k += 1;
            }
            if used {
                let first =
                    u64::from_le_bytes(BLK_DATA_PAGE.0[off + 32..off + 40].try_into().unwrap());
                let last =
                    u64::from_le_bytes(BLK_DATA_PAGE.0[off + 40..off + 48].try_into().unwrap());
                serial_write(b"GPT: part first=0x");
                serial_write_hex(first);
                serial_write(b" last=0x");
                serial_write_hex(last);
                serial_write(b"\n");
                count += 1;
            }
            j += 1;
            e += 1;
        }
    }
    serial_write(b"GPT: parsed n=0x");
    serial_write_hex(count);
    serial_write(b"\n");
}

/// IEEE 802.3 CRC-32 (reflected, polynomial 0xEDB88320) over `data`. Bitwise (no
/// table) to keep the .text small. The same CRC GPT uses for its header and
/// partition-array integrity fields.
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut i = 0;
    while i < data.len() {
        crc ^= data[i] as u32;
        let mut k = 0;
        while k < 8 {
            let m = (crc & 1).wrapping_neg(); // 0xFFFFFFFF when the low bit is set
            crc = (crc >> 1) ^ (0xEDB8_8320 & m);
            k += 1;
        }
        i += 1;
    }
    !crc
}

/// GPT header-CRC validation self-test (full-os guide Part II.5): proves the CRC
/// machinery a full GPT validator needs — (1) a known-answer test that crc32 is
/// the real IEEE CRC-32 (CRC32("123456789") == 0xCBF43926); (2) synthesize a GPT
/// header, stamp its header CRC at offset 16, and validate it the standard GPT way
/// (zero the CRC field, recompute over HeaderSize bytes, compare); (3) flip a byte
/// and confirm the recomputed CRC no longer matches. Marker `GPT: hdr crc ok`.
pub(crate) unsafe fn gpt_crc_selftest() {
    // (1) Known-answer test: the standard CRC-32 check value.
    let kat = crc32(b"123456789") == 0xCBF4_3926;

    // (2) Synthesize a minimal GPT header and stamp its header CRC (offset 16..20).
    const HDR_SIZE: usize = 92;
    let mut hdr = [0u8; HDR_SIZE];
    hdr[0..8].copy_from_slice(b"EFI PART");
    hdr[8..12].copy_from_slice(&0x0001_0000u32.to_le_bytes()); // revision 1.0
    hdr[12..16].copy_from_slice(&(HDR_SIZE as u32).to_le_bytes()); // HeaderSize
    // offset 16..20 = header CRC, left zero while computing
    hdr[24..32].copy_from_slice(&1u64.to_le_bytes()); // MyLBA = 1
    let stored = crc32(&hdr[..HDR_SIZE]);
    hdr[16..20].copy_from_slice(&stored.to_le_bytes());

    // Validate as a real GPT consumer would: read the stored CRC, zero the field,
    // recompute over HeaderSize bytes, and compare.
    let read_crc = u32::from_le_bytes(hdr[16..20].try_into().unwrap());
    let mut tmp = hdr;
    tmp[16..20].copy_from_slice(&[0u8; 4]);
    let valid = crc32(&tmp[..HDR_SIZE]) == read_crc;

    // (3) Corrupt one header byte: the recomputed CRC must no longer match.
    let mut bad = hdr;
    bad[24] ^= 0xFF;
    bad[16..20].copy_from_slice(&[0u8; 4]);
    let rejects = crc32(&bad[..HDR_SIZE]) != read_crc;

    if kat && valid && rejects {
        serial_write(b"GPT: hdr crc ok\n");
    } else {
        serial_write(b"GPT: hdr crc fail\n");
    }
}
