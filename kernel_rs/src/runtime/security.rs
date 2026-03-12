pub const HANDLE_RIGHT_READ: u64 = 1 << 0;
pub const HANDLE_RIGHT_WRITE: u64 = 1 << 1;
pub const HANDLE_RIGHT_POLL: u64 = 1 << 2;
pub const HANDLE_RIGHT_MASK: u64 =
    HANDLE_RIGHT_READ | HANDLE_RIGHT_WRITE | HANDLE_RIGHT_POLL;

pub fn clamp_rights(rights: u64) -> u64 {
    rights & HANDLE_RIGHT_MASK
}

pub fn monotonic_rights(current: u64, requested: u64) -> Option<u64> {
    let next = clamp_rights(requested);
    if next & !current != 0 {
        return None;
    }
    Some(next)
}

pub fn requested_open_rights(
    flags: u64,
    mode_mask: u64,
    rdonly: u64,
    wronly: u64,
    rdwr: u64,
) -> Option<u64> {
    if flags & !mode_mask != 0 {
        return None;
    }
    match flags & mode_mask {
        mode if mode == rdonly => Some(HANDLE_RIGHT_READ),
        mode if mode == wronly => Some(HANDLE_RIGHT_WRITE),
        mode if mode == rdwr => Some(HANDLE_RIGHT_READ | HANDLE_RIGHT_WRITE),
        _ => None,
    }
}
