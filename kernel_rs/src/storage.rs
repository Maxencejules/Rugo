// Durable runtime state for the default Go lane.

use crate::*;

#[cfg(feature = "go_test")]
const R4_STORAGE_JOURNAL_MAGIC: u32 = 0x4A52_4E31;
#[cfg(feature = "go_test")]
const R4_STORAGE_STATE_MAGIC: u32 = 0x5354_4131;
#[cfg(feature = "go_test")]
const R4_STORAGE_JOURNAL_SECTOR: u64 = 8;
#[cfg(feature = "go_test")]
const R4_STORAGE_STATE_SECTOR: u64 = 9;
#[cfg(feature = "go_test")]
const R4_STORAGE_MAX_BYTES: usize = 480;
#[cfg(feature = "go_test")]
pub(crate) const R4_STORAGE_RUNTIME_FILE_MAX_BYTES: usize = 64;
#[cfg(feature = "go_test")]
const R4_STORAGE_RUNTIME_FILE_COUNT: usize = 2;
#[cfg(feature = "go_test")]
const R4_STORAGE_PKGSTATE_MAGIC: u32 = 0x504B_4731;
#[cfg(feature = "go_test")]
const R4_STORAGE_PLATFORM_MAGIC: u32 = 0x5046_5431;
#[cfg(feature = "go_test")]
const R4_STORAGE_PKGSTATE_SECTOR: u64 = 10;
#[cfg(feature = "go_test")]
const R4_STORAGE_PLATFORM_SECTOR: u64 = 11;

#[cfg(feature = "go_test")]
static mut R4_STORAGE_READY: bool = false;
#[cfg(feature = "go_test")]
static mut R4_STORAGE_RECOVERED: bool = false;
#[cfg(feature = "go_test")]
static mut R4_STORAGE_SEQ: u32 = 0;
#[cfg(feature = "go_test")]
static mut R4_STORAGE_DURABLE_LEN: usize = 0;
#[cfg(feature = "go_test")]
static mut R4_STORAGE_DURABLE: [u8; R4_STORAGE_MAX_BYTES] = [0; R4_STORAGE_MAX_BYTES];
#[cfg(feature = "go_test")]
static mut R4_STORAGE_JOURNAL_LEN: usize = 0;
#[cfg(feature = "go_test")]
static mut R4_STORAGE_JOURNAL: [u8; R4_STORAGE_MAX_BYTES] = [0; R4_STORAGE_MAX_BYTES];
#[cfg(feature = "go_test")]
static mut R4_STORAGE_RUNTIME_FILE_LEN: [usize; R4_STORAGE_RUNTIME_FILE_COUNT] =
    [0; R4_STORAGE_RUNTIME_FILE_COUNT];
#[cfg(feature = "go_test")]
static mut R4_STORAGE_RUNTIME_FILE_SEQ: [u32; R4_STORAGE_RUNTIME_FILE_COUNT] =
    [0; R4_STORAGE_RUNTIME_FILE_COUNT];
#[cfg(feature = "go_test")]
static mut R4_STORAGE_RUNTIME_FILE_CACHE: [[u8; R4_STORAGE_RUNTIME_FILE_MAX_BYTES];
    R4_STORAGE_RUNTIME_FILE_COUNT] = [[0; R4_STORAGE_RUNTIME_FILE_MAX_BYTES];
    R4_STORAGE_RUNTIME_FILE_COUNT];

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_available() -> bool {
    R4_STORAGE_READY
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_state_len() -> usize {
    R4_STORAGE_DURABLE_LEN
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_copy_state(offset: usize, dst: &mut [u8]) -> bool {
    if offset > R4_STORAGE_DURABLE_LEN || offset + dst.len() > R4_STORAGE_DURABLE_LEN {
        return false;
    }
    dst.copy_from_slice(&R4_STORAGE_DURABLE[offset..offset + dst.len()]);
    true
}

#[cfg(feature = "go_test")]
#[derive(Clone, Copy)]
pub(crate) enum R4StorageRuntimeFile {
    PkgState = 0,
    Platform = 1,
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_runtime_magic(file: R4StorageRuntimeFile) -> u32 {
    match file {
        R4StorageRuntimeFile::PkgState => R4_STORAGE_PKGSTATE_MAGIC,
        R4StorageRuntimeFile::Platform => R4_STORAGE_PLATFORM_MAGIC,
    }
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_runtime_sector(file: R4StorageRuntimeFile) -> u64 {
    match file {
        R4StorageRuntimeFile::PkgState => R4_STORAGE_PKGSTATE_SECTOR,
        R4StorageRuntimeFile::Platform => R4_STORAGE_PLATFORM_SECTOR,
    }
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_runtime_index(file: R4StorageRuntimeFile) -> usize {
    file as usize
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_runtime_len(file: R4StorageRuntimeFile) -> usize {
    R4_STORAGE_RUNTIME_FILE_LEN[r4_storage_runtime_index(file)]
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_runtime_copy(
    file: R4StorageRuntimeFile,
    offset: usize,
    dst: &mut [u8],
) -> bool {
    let idx = r4_storage_runtime_index(file);
    let total = R4_STORAGE_RUNTIME_FILE_LEN[idx];
    if offset > total || offset + dst.len() > total {
        return false;
    }
    dst.copy_from_slice(&R4_STORAGE_RUNTIME_FILE_CACHE[idx][offset..offset + dst.len()]);
    true
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_runtime_load(file: R4StorageRuntimeFile) {
    let idx = r4_storage_runtime_index(file);
    R4_STORAGE_RUNTIME_FILE_LEN[idx] = 0;
    R4_STORAGE_RUNTIME_FILE_SEQ[idx] = 0;
    core::ptr::write_bytes(
        R4_STORAGE_RUNTIME_FILE_CACHE[idx].as_mut_ptr(),
        0,
        R4_STORAGE_RUNTIME_FILE_CACHE[idx].len(),
    );
    if !block_io_dispatch(false, r4_storage_runtime_sector(file), 512, false) {
        return;
    }
    let magic = u32::from_le_bytes([
        BLK_DATA_PAGE.0[0],
        BLK_DATA_PAGE.0[1],
        BLK_DATA_PAGE.0[2],
        BLK_DATA_PAGE.0[3],
    ]);
    if magic != r4_storage_runtime_magic(file) {
        return;
    }
    let len = u32::from_le_bytes([
        BLK_DATA_PAGE.0[8],
        BLK_DATA_PAGE.0[9],
        BLK_DATA_PAGE.0[10],
        BLK_DATA_PAGE.0[11],
    ]) as usize;
    if len > R4_STORAGE_RUNTIME_FILE_MAX_BYTES {
        return;
    }
    let seq = u32::from_le_bytes([
        BLK_DATA_PAGE.0[12],
        BLK_DATA_PAGE.0[13],
        BLK_DATA_PAGE.0[14],
        BLK_DATA_PAGE.0[15],
    ]);
    R4_STORAGE_RUNTIME_FILE_LEN[idx] = len;
    R4_STORAGE_RUNTIME_FILE_SEQ[idx] = seq;
    if len != 0 {
        R4_STORAGE_RUNTIME_FILE_CACHE[idx][..len]
            .copy_from_slice(&BLK_DATA_PAGE.0[16..16 + len]);
    }
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_runtime_write(file: R4StorageRuntimeFile, data: &[u8]) -> bool {
    if !R4_STORAGE_READY || data.len() > R4_STORAGE_RUNTIME_FILE_MAX_BYTES {
        return false;
    }
    let idx = r4_storage_runtime_index(file);
    let seq = R4_STORAGE_RUNTIME_FILE_SEQ[idx].wrapping_add(1);
    if !r4_storage_write_record(
        r4_storage_runtime_sector(file),
        r4_storage_runtime_magic(file),
        0,
        seq,
        data,
        matches!(ACTIVE_BLOCK_DRIVER, ActiveBlockDriver::Nvme),
    ) {
        return false;
    }
    R4_STORAGE_RUNTIME_FILE_SEQ[idx] = seq;
    R4_STORAGE_RUNTIME_FILE_LEN[idx] = data.len();
    core::ptr::write_bytes(
        R4_STORAGE_RUNTIME_FILE_CACHE[idx].as_mut_ptr(),
        0,
        R4_STORAGE_RUNTIME_FILE_CACHE[idx].len(),
    );
    if !data.is_empty() {
        R4_STORAGE_RUNTIME_FILE_CACHE[idx][..data.len()].copy_from_slice(data);
    }
    true
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_reset_cache() {
    R4_STORAGE_READY = false;
    R4_STORAGE_RECOVERED = false;
    R4_STORAGE_SEQ = 0;
    R4_STORAGE_DURABLE_LEN = 0;
    R4_STORAGE_JOURNAL_LEN = 0;
    core::ptr::write_bytes(R4_STORAGE_DURABLE.as_mut_ptr(), 0, R4_STORAGE_DURABLE.len());
    core::ptr::write_bytes(R4_STORAGE_JOURNAL.as_mut_ptr(), 0, R4_STORAGE_JOURNAL.len());
    let mut idx = 0usize;
    while idx < R4_STORAGE_RUNTIME_FILE_COUNT {
        R4_STORAGE_RUNTIME_FILE_LEN[idx] = 0;
        R4_STORAGE_RUNTIME_FILE_SEQ[idx] = 0;
        core::ptr::write_bytes(
            R4_STORAGE_RUNTIME_FILE_CACHE[idx].as_mut_ptr(),
            0,
            R4_STORAGE_RUNTIME_FILE_CACHE[idx].len(),
        );
        idx += 1;
    }
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_write_record(
    sector: u64,
    magic: u32,
    flags: u32,
    seq: u32,
    data: &[u8],
    fua: bool,
) -> bool {
    if data.len() > R4_STORAGE_MAX_BYTES {
        return false;
    }
    core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
    BLK_DATA_PAGE.0[0..4].copy_from_slice(&magic.to_le_bytes());
    BLK_DATA_PAGE.0[4..8].copy_from_slice(&flags.to_le_bytes());
    BLK_DATA_PAGE.0[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
    BLK_DATA_PAGE.0[12..16].copy_from_slice(&seq.to_le_bytes());
    if !data.is_empty() {
        BLK_DATA_PAGE.0[16..16 + data.len()].copy_from_slice(data);
    }
    block_io_dispatch(true, sector, 512, fua)
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_load_state_cache() {
    R4_STORAGE_DURABLE_LEN = 0;
    if !block_io_dispatch(false, R4_STORAGE_STATE_SECTOR, 512, false) {
        return;
    }
    let magic = u32::from_le_bytes([
        BLK_DATA_PAGE.0[0],
        BLK_DATA_PAGE.0[1],
        BLK_DATA_PAGE.0[2],
        BLK_DATA_PAGE.0[3],
    ]);
    if magic != R4_STORAGE_STATE_MAGIC {
        return;
    }
    let len = u32::from_le_bytes([
        BLK_DATA_PAGE.0[8],
        BLK_DATA_PAGE.0[9],
        BLK_DATA_PAGE.0[10],
        BLK_DATA_PAGE.0[11],
    ]) as usize;
    if len > R4_STORAGE_MAX_BYTES {
        return;
    }
    let seq = u32::from_le_bytes([
        BLK_DATA_PAGE.0[12],
        BLK_DATA_PAGE.0[13],
        BLK_DATA_PAGE.0[14],
        BLK_DATA_PAGE.0[15],
    ]);
    R4_STORAGE_SEQ = seq;
    R4_STORAGE_DURABLE_LEN = len;
    if len != 0 {
        R4_STORAGE_DURABLE[..len].copy_from_slice(&BLK_DATA_PAGE.0[16..16 + len]);
    }
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_clear_journal() -> bool {
    R4_STORAGE_JOURNAL_LEN = 0;
    core::ptr::write_bytes(R4_STORAGE_JOURNAL.as_mut_ptr(), 0, R4_STORAGE_JOURNAL.len());
    r4_storage_write_record(
        R4_STORAGE_JOURNAL_SECTOR,
        R4_STORAGE_JOURNAL_MAGIC,
        0,
        R4_STORAGE_SEQ,
        &[],
        matches!(ACTIVE_BLOCK_DRIVER, ActiveBlockDriver::Nvme),
    )
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_commit_state(data: &[u8]) -> bool {
    if data.len() > R4_STORAGE_MAX_BYTES {
        return false;
    }
    if !r4_storage_write_record(
        R4_STORAGE_STATE_SECTOR,
        R4_STORAGE_STATE_MAGIC,
        0,
        R4_STORAGE_SEQ,
        data,
        matches!(ACTIVE_BLOCK_DRIVER, ActiveBlockDriver::Nvme),
    ) {
        return false;
    }
    R4_STORAGE_DURABLE_LEN = data.len();
    if !data.is_empty() {
        R4_STORAGE_DURABLE[..data.len()].copy_from_slice(data);
    }
    true
}

#[cfg(feature = "go_test")]
unsafe fn r4_storage_boot_recover() {
    if !block_io_dispatch(false, R4_STORAGE_JOURNAL_SECTOR, 512, false) {
        r4_storage_load_state_cache();
        return;
    }
    let magic = u32::from_le_bytes([
        BLK_DATA_PAGE.0[0],
        BLK_DATA_PAGE.0[1],
        BLK_DATA_PAGE.0[2],
        BLK_DATA_PAGE.0[3],
    ]);
    let flags = u32::from_le_bytes([
        BLK_DATA_PAGE.0[4],
        BLK_DATA_PAGE.0[5],
        BLK_DATA_PAGE.0[6],
        BLK_DATA_PAGE.0[7],
    ]);
    let len = u32::from_le_bytes([
        BLK_DATA_PAGE.0[8],
        BLK_DATA_PAGE.0[9],
        BLK_DATA_PAGE.0[10],
        BLK_DATA_PAGE.0[11],
    ]) as usize;
    let seq = u32::from_le_bytes([
        BLK_DATA_PAGE.0[12],
        BLK_DATA_PAGE.0[13],
        BLK_DATA_PAGE.0[14],
        BLK_DATA_PAGE.0[15],
    ]);
    if seq > R4_STORAGE_SEQ {
        R4_STORAGE_SEQ = seq;
    }
    if magic == R4_STORAGE_JOURNAL_MAGIC && flags == 1 && len <= R4_STORAGE_MAX_BYTES {
        R4_STORAGE_JOURNAL_LEN = len;
        if len != 0 {
            R4_STORAGE_JOURNAL[..len].copy_from_slice(&BLK_DATA_PAGE.0[16..16 + len]);
        }
        if r4_storage_commit_state(&R4_STORAGE_JOURNAL[..len]) && r4_storage_clear_journal() {
            R4_STORAGE_RECOVERED = true;
            serial_write(b"RECOV: replay ok\n");
        }
    }
    r4_storage_load_state_cache();
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_write_journal(data: &[u8]) -> bool {
    if !R4_STORAGE_READY || data.is_empty() || data.len() > R4_STORAGE_MAX_BYTES {
        return false;
    }
    R4_STORAGE_SEQ = R4_STORAGE_SEQ.wrapping_add(1);
    R4_STORAGE_JOURNAL_LEN = data.len();
    R4_STORAGE_JOURNAL[..data.len()].copy_from_slice(data);
    r4_storage_write_record(
        R4_STORAGE_JOURNAL_SECTOR,
        R4_STORAGE_JOURNAL_MAGIC,
        1,
        R4_STORAGE_SEQ,
        data,
        false,
    )
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_fsync() -> bool {
    if !R4_STORAGE_READY || R4_STORAGE_JOURNAL_LEN == 0 {
        return false;
    }
    let len = R4_STORAGE_JOURNAL_LEN;
    if !r4_storage_commit_state(&R4_STORAGE_JOURNAL[..len]) {
        return false;
    }
    if !r4_storage_clear_journal() {
        return false;
    }
    if matches!(ACTIVE_BLOCK_DRIVER, ActiveBlockDriver::Nvme) {
        serial_write(b"BLK: fua ok\n");
    }
    if !block_flush_dispatch() {
        return false;
    }
    serial_write(b"BLK: flush ordered\n");
    true
}

#[cfg(feature = "go_test")]
pub(crate) unsafe fn r4_storage_boot_probe() {
    r4_storage_reset_cache();
    let hhdm_resp_ptr = core::ptr::read_volatile(core::ptr::addr_of!(HHDM_REQUEST.response));
    let kaddr_resp_ptr = core::ptr::read_volatile(core::ptr::addr_of!(KADDR_REQUEST.response));
    let kphys = (*kaddr_resp_ptr).physical_base;
    let kvirt = (*kaddr_resp_ptr).virtual_base;
    HHDM_OFFSET = (*hhdm_resp_ptr).offset;
    BLK_KV2P_DELTA = kphys.wrapping_sub(kvirt);

    if block_driver_probe(true, cfg!(feature = "native_go_test"), cfg!(feature = "native_go_test"))
    {
        R4_STORAGE_READY = true;
        serial_write(b"STORC4: block ready driver=");
        serial_write(block_driver_class());
        serial_write(b"\n");
        r4_storage_boot_recover();
        r4_storage_runtime_load(R4StorageRuntimeFile::PkgState);
        r4_storage_runtime_load(R4StorageRuntimeFile::Platform);
        #[cfg(not(feature = "compat_real_test"))]
        crate::vfs::vfs_mount();
    }
}
