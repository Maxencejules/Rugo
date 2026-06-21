//! FAT16 on-disk parser (full-os guide Part II.5): read (single cluster + chain),
//! single-cluster write, and root-directory list.
//!
//! Extracted from `lib.rs` (gap #9, maintainability). This is the pure on-disk
//! parser; the `/mnt` `FAT_FILE` cache and its fd integration stay in lib.rs and
//! call these via `crate::fat16::`. Deps are reached via `crate::` paths
//! (descendant access — no `pub(crate)` widening): the shared block scratch
//! buffer `BLK_DATA_PAGE`, `block_io_dispatch`, `storage`, and `serial_write*`.

use crate::{block_io_dispatch, serial_write, serial_write_hex, storage, BLK_DATA_PAGE};

/// Read a file by its 11-byte 8.3 directory name from the FAT16 volume at a
/// fixed LBA into `out` (single cluster, v1). Returns the byte count, or None
/// if the volume/BPB is bad or the file is absent (full-os guide Part II.5 FAT;
/// shared by `sys_sysinfo` op 6 and the `/mnt` open path).
pub(crate) unsafe fn fat16_read_named(target: &[u8; 11], out: &mut [u8]) -> Option<usize> {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || !block_io_dispatch(false, VOL_LBA, 512, false) {
        return None;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let spc = BLK_DATA_PAGE.0[13] as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || spc == 0 || nfats == 0 {
        return None;
    }
    let root_lba = VOL_LBA + reserved + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let data_lba = root_lba + root_sectors;
    let mut first_cluster = 0u64;
    let mut file_size = 0u64;
    let mut s = 0u64;
    'scan: while s < root_sectors {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return None;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            if BLK_DATA_PAGE.0[base] == 0 {
                break 'scan;
            }
            if BLK_DATA_PAGE.0[base..base + 11] == *target {
                first_cluster = u16::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 26],
                    BLK_DATA_PAGE.0[base + 27],
                ]) as u64;
                file_size = u32::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 28],
                    BLK_DATA_PAGE.0[base + 29],
                    BLK_DATA_PAGE.0[base + 30],
                    BLK_DATA_PAGE.0[base + 31],
                ]) as u64;
                break 'scan;
            }
            e += 1;
        }
        s += 1;
    }
    if first_cluster < 2 || file_size == 0 {
        return None;
    }
    let cluster_lba = data_lba + (first_cluster - 2) * spc;
    if !block_io_dispatch(false, cluster_lba, 512, false) {
        return None;
    }
    let n = core::cmp::min(file_size as usize, out.len()).min(512);
    out[..n].copy_from_slice(&BLK_DATA_PAGE.0[..n]);
    Some(n)
}

/// Read a file by its 8.3 name following the FAT16 cluster CHAIN (full-os guide
/// Part II.5): unlike `fat16_read_named` (first cluster only), this walks the FAT
/// from the directory's first cluster to the end-of-chain marker, so it reads
/// files spanning multiple clusters. Returns min(file_size, out.len()) bytes, or
/// None on a bad volume/absent file/corrupt chain.
pub(crate) unsafe fn fat16_read_chain(target: &[u8; 11], out: &mut [u8]) -> Option<usize> {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || !block_io_dispatch(false, VOL_LBA, 512, false) {
        return None;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let spc = BLK_DATA_PAGE.0[13] as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || spc == 0 || nfats == 0 {
        return None;
    }
    let fat_lba = VOL_LBA + reserved;
    let root_lba = fat_lba + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let data_lba = root_lba + root_sectors;

    // Locate the directory entry: first cluster + file size.
    let mut cluster = 0u64;
    let mut file_size = 0u64;
    let mut s = 0u64;
    'scan: while s < root_sectors {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return None;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            if BLK_DATA_PAGE.0[base] == 0 {
                break 'scan;
            }
            if BLK_DATA_PAGE.0[base..base + 11] == *target {
                cluster =
                    u16::from_le_bytes([BLK_DATA_PAGE.0[base + 26], BLK_DATA_PAGE.0[base + 27]])
                        as u64;
                file_size = u32::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 28],
                    BLK_DATA_PAGE.0[base + 29],
                    BLK_DATA_PAGE.0[base + 30],
                    BLK_DATA_PAGE.0[base + 31],
                ]) as u64;
                break 'scan;
            }
            e += 1;
        }
        s += 1;
    }
    if cluster < 2 || file_size == 0 {
        return None;
    }

    // Walk the chain: copy each cluster's sectors, then follow the FAT16 entry
    // (2 bytes per cluster) to the next cluster until a terminal value. A valid data
    // cluster is [2, 0xFFEF]; 0xFFF0..=0xFFF6 are reserved, 0xFFF7 is a bad-cluster
    // marker, and 0xFFF8..=0xFFFF is end-of-chain. Stopping at `< 0xFFF0` means a
    // reserved or bad-cluster FAT entry terminates the read instead of being followed
    // as a (bogus, out-of-range) cluster number -- which would compute a wild
    // data_lba and read garbage off the disk. The iteration guard additionally bounds
    // a corrupt self-referential chain so it cannot loop forever.
    let want = core::cmp::min(file_size as usize, out.len());
    let mut written = 0usize;
    let mut guard = 0u64;
    let max_clusters = out.len() as u64 / (spc * 512) + 4;
    while cluster >= 2 && cluster < 0xFFF0 && written < want {
        guard += 1;
        if guard > max_clusters {
            return None;
        }
        let cluster_lba = data_lba + (cluster - 2) * spc;
        let mut sec = 0u64;
        while sec < spc && written < want {
            if !block_io_dispatch(false, cluster_lba + sec, 512, false) {
                return None;
            }
            let n = core::cmp::min(512, want - written);
            out[written..written + n].copy_from_slice(&BLK_DATA_PAGE.0[..n]);
            written += n;
            sec += 1;
        }
        // Read the FAT entry for `cluster` (this overwrites BLK_DATA_PAGE, but the
        // cluster data was already copied into `out`).
        let fat_byte = cluster * 2;
        let fat_sec = fat_lba + fat_byte / 512;
        let off = (fat_byte % 512) as usize;
        if !block_io_dispatch(false, fat_sec, 512, false) {
            return None;
        }
        cluster = u16::from_le_bytes([BLK_DATA_PAGE.0[off], BLK_DATA_PAGE.0[off + 1]]) as u64;
    }
    Some(written)
}

/// Write a single-cluster file to the FAT16 root directory (full-os guide Part
/// II.5, FS maturity): allocate the first free cluster, mark it EOC in every FAT
/// copy, write the data into that cluster, and fill a free root-dir 8.3 entry.
/// v1 boundary: one cluster (≤512 B), root dir only, no overwrite/append/chain.
/// Returns true on success. Shares the BPB parse with `fat16_read_named`.
pub(crate) unsafe fn fat16_write_named(target: &[u8; 11], data: &[u8]) -> bool {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || data.is_empty() || data.len() > 512 {
        return false;
    }
    if !block_io_dispatch(false, VOL_LBA, 512, false) {
        return false;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let spc = BLK_DATA_PAGE.0[13] as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || spc == 0 || nfats == 0 || spf == 0 {
        return false;
    }
    let fat_lba = VOL_LBA + reserved;
    let root_lba = fat_lba + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let data_lba = root_lba + root_sectors;

    // 1) Find a free root-dir slot AND reject a duplicate name -- BEFORE any
    // on-disk mutation, so a full directory or an already-present name leaks
    // nothing (the deterministic leak the review found). Scan to the first 0x00
    // (end-of-directory): entries beyond it are unused.
    let mut slot_lba = 0u64;
    let mut slot_off = 0usize;
    let mut found_slot = false;
    let mut ended = false;
    let mut s = 0u64;
    while s < root_sectors && !ended {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return false;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            let b0 = BLK_DATA_PAGE.0[base];
            if b0 == 0x00 {
                // End of directory: the first free slot if none seen earlier.
                if !found_slot {
                    slot_lba = root_lba + s;
                    slot_off = base;
                    found_slot = true;
                }
                ended = true;
                break;
            }
            if b0 != 0xE5 && BLK_DATA_PAGE.0[base..base + 11] == *target {
                return false; // name already exists -> refuse (no duplicate entry)
            }
            if b0 == 0xE5 && !found_slot {
                slot_lba = root_lba + s;
                slot_off = base;
                found_slot = true;
            }
            e += 1;
        }
        s += 1;
    }
    if !found_slot {
        return false; // directory full -> nothing committed
    }
    // 2) Find a free cluster (first FAT sector, clusters 2..255) -- still no
    // mutation, so a full FAT also leaks nothing.
    if !block_io_dispatch(false, fat_lba, 512, false) {
        return false;
    }
    let mut cluster = 0u64;
    let mut c = 2usize;
    while c < 256 {
        if u16::from_le_bytes([BLK_DATA_PAGE.0[c * 2], BLK_DATA_PAGE.0[c * 2 + 1]]) == 0 {
            cluster = c as u64;
            break;
        }
        c += 1;
    }
    if cluster < 2 {
        return false; // no free cluster -> nothing committed
    }
    // 3) Commit: mark the cluster end-of-chain in every FAT copy, write the data
    // cluster, then link it from the directory entry. (A mid-commit device-write
    // failure can still leave an orphaned cluster or divergent FAT copies; that
    // is inherent to a non-journaling FAT writer -- see the v1 boundary.)
    BLK_DATA_PAGE.0[(cluster as usize) * 2] = 0xFF;
    BLK_DATA_PAGE.0[(cluster as usize) * 2 + 1] = 0xFF;
    let mut f = 0u64;
    while f < nfats {
        if !block_io_dispatch(true, fat_lba + f * spf, 512, false) {
            return false;
        }
        f += 1;
    }
    let cluster_lba = data_lba + (cluster - 2) * spc;
    core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
    BLK_DATA_PAGE.0[..data.len()].copy_from_slice(data);
    if !block_io_dispatch(true, cluster_lba, 512, false) {
        return false;
    }
    // Re-read the chosen directory sector (the buffer was reused above) and fill
    // the slot we reserved in step 1.
    if !block_io_dispatch(false, slot_lba, 512, false) {
        return false;
    }
    core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr().add(slot_off), 0, 32);
    BLK_DATA_PAGE.0[slot_off..slot_off + 11].copy_from_slice(target);
    BLK_DATA_PAGE.0[slot_off + 11] = 0x20; // attr = archive
    BLK_DATA_PAGE.0[slot_off + 26] = (cluster & 0xFF) as u8;
    BLK_DATA_PAGE.0[slot_off + 27] = ((cluster >> 8) & 0xFF) as u8;
    BLK_DATA_PAGE.0[slot_off + 28..slot_off + 32]
        .copy_from_slice(&(data.len() as u32).to_le_bytes());
    block_io_dispatch(true, slot_lba, 512, false)
}

/// List the FAT16 root directory (full-os guide Part II.5): log each live 8.3
/// entry as `FATLS: <name11> size=0x<hex>` and return the count, or u64::MAX on
/// a bad volume. Skips free (0x00), deleted (0xE5), and long-name (attr 0x0F)
/// entries. Shares the BPB parse with `fat16_read_named`.
pub(crate) unsafe fn fat16_list() -> u64 {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || !block_io_dispatch(false, VOL_LBA, 512, false) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || nfats == 0 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let root_lba = VOL_LBA + reserved + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let mut count = 0u64;
    let mut s = 0u64;
    'scan: while s < root_sectors {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            let first = BLK_DATA_PAGE.0[base];
            if first == 0x00 {
                break 'scan; // end of directory
            }
            // Skip deleted (0xE5) and long-file-name (attr 0x0F) entries.
            if first != 0xE5 && BLK_DATA_PAGE.0[base + 11] != 0x0F {
                let size = u32::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 28],
                    BLK_DATA_PAGE.0[base + 29],
                    BLK_DATA_PAGE.0[base + 30],
                    BLK_DATA_PAGE.0[base + 31],
                ]);
                serial_write(b"FATLS: ");
                serial_write(&BLK_DATA_PAGE.0[base..base + 11]);
                serial_write(b" size=0x");
                serial_write_hex(size as u64);
                serial_write(b"\n");
                count += 1;
            }
            e += 1;
        }
        s += 1;
    }
    count
}
