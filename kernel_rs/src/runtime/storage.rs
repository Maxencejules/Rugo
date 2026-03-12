pub const SIMPLEFS_MAGIC: u32 = 0x5346_5331;
pub const PKG_MAGIC_V1: u32 = 0x0147_4B50;

pub fn block_io_len_valid(len: u64) -> bool {
    len != 0 && len <= 4096 && len % 512 == 0
}
