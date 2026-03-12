pub fn under_quota(current: usize, max: usize) -> bool {
    current < max
}

pub fn owner_has_right(
    owner_tid: usize,
    current_tid: usize,
    owner_rights: u8,
    required_right: u8,
) -> bool {
    owner_tid == current_tid && (owner_rights & required_right) != 0
}
