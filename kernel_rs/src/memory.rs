// User virtual-address validation and copy helpers.

use crate::HHDM_OFFSET;

pub(crate) const USER_VA_LIMIT: u64 = 0x0000_8000_0000_0000;
pub(crate) const USER_PERM_READ: u64 = 1 << 0;
pub(crate) const USER_PERM_WRITE: u64 = 1 << 1;
const USER_COPYINSTR_MAX: usize = 256;

#[allow(dead_code)]
pub(crate) struct Vec<T> {
    len: usize,
    buf: [u8; USER_COPYINSTR_MAX],
    _marker: core::marker::PhantomData<T>,
}

impl Vec<u8> {
    fn new() -> Self {
        Self {
            len: 0,
            buf: [0u8; USER_COPYINSTR_MAX],
            _marker: core::marker::PhantomData,
        }
    }

    fn push(&mut self, b: u8) -> Result<(), ()> {
        if self.len >= self.buf.len() {
            return Err(());
        }
        self.buf[self.len] = b;
        self.len += 1;
        Ok(())
    }

    fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl core::ops::Deref for Vec<u8> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

pub(crate) unsafe fn check_page_user_perms(va: u64, hhdm: u64, required_perms: u64) -> bool {
    let need_write = (required_perms & USER_PERM_WRITE) != 0;
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let pml4_phys = cr3 & 0x000F_FFFF_FFFF_F000;
    let pml4 = (pml4_phys + hhdm) as *const u64;
    let pml4e = *pml4.add(((va >> 39) & 0x1FF) as usize);
    if pml4e & 1 == 0 || pml4e & 4 == 0 {
        return false;
    }
    if need_write && pml4e & 2 == 0 {
        return false;
    }
    let pdpt = ((pml4e & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
    let pdpte = *pdpt.add(((va >> 30) & 0x1FF) as usize);
    if pdpte & 1 == 0 || pdpte & 4 == 0 {
        return false;
    }
    if need_write && pdpte & 2 == 0 {
        return false;
    }
    if pdpte & 0x80 != 0 {
        return true;
    }
    let pd = ((pdpte & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
    let pde = *pd.add(((va >> 21) & 0x1FF) as usize);
    if pde & 1 == 0 || pde & 4 == 0 {
        return false;
    }
    if need_write && pde & 2 == 0 {
        return false;
    }
    if pde & 0x80 != 0 {
        return true;
    }
    let pt = ((pde & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
    let pte = *pt.add(((va >> 12) & 0x1FF) as usize);
    if pte & 1 == 0 || pte & 4 == 0 {
        return false;
    }
    // A copy-on-write page (PTE_COW, bit 9) reads as non-writable but is
    // writable-on-write: a kernel/user write will break it. Accept it for
    // WRITE; copyout_user breaks it before the actual store.
    if need_write && pte & 2 == 0 && pte & 0x200 == 0 {
        return false;
    }
    true
}

pub(crate) fn user_range_ok(ptr: u64, len: usize) -> bool {
    if len == 0 {
        return true;
    }
    match ptr.checked_add(len as u64) {
        Some(end) => ptr < USER_VA_LIMIT && end <= USER_VA_LIMIT,
        None => false,
    }
}

pub(crate) unsafe fn user_pages_ok(ptr: u64, len: usize, required_perms: u64) -> bool {
    if len == 0 {
        return true;
    }
    if !user_range_ok(ptr, len) {
        return false;
    }
    let hhdm = HHDM_OFFSET;
    let start_page = ptr & !0xFFF;
    let end = match ptr.checked_add(len as u64 - 1) {
        Some(v) => v,
        None => return false,
    };
    let end_page = end & !0xFFF;
    let mut page = start_page;
    loop {
        if !check_page_user_perms(page, hhdm, required_perms) {
            // Demand-window pages may not have faulted in yet: user code
            // can pass freshly allocated, never-touched heap memory
            // straight into a syscall. Pre-map them here exactly as the
            // page-fault path would, then re-check.
            let premapped = page >= crate::mm::DEMAND_BASE
                && page < crate::mm::DEMAND_END
                && crate::mm::try_demand_map(page)
                && check_page_user_perms(page, hhdm, required_perms);
            if !premapped {
                return false;
            }
        }
        if page >= end_page {
            break;
        }
        page += 4096;
    }
    true
}

pub(crate) unsafe fn copyin_user(dst: &mut [u8], user_ptr: u64, len: usize) -> Result<(), ()> {
    if len > dst.len() {
        return Err(());
    }
    if !user_range_ok(user_ptr, len) {
        return Err(());
    }
    if !user_pages_ok(user_ptr, len, USER_PERM_READ) {
        return Err(());
    }
    if len > 0 {
        core::ptr::copy_nonoverlapping(user_ptr as *const u8, dst.as_mut_ptr(), len);
    }
    Ok(())
}

pub(crate) unsafe fn copyout_user(user_ptr: u64, src: &[u8], len: usize) -> Result<(), ()> {
    if len > src.len() {
        return Err(());
    }
    if !user_range_ok(user_ptr, len) {
        return Err(());
    }
    if !user_pages_ok(user_ptr, len, USER_PERM_WRITE) {
        return Err(());
    }
    if len > 0 {
        // Break copy-on-write across the destination so the store targets a
        // private, writable frame instead of faulting on (or silently
        // corrupting) a shared read-only CoW page. cow_break is a no-op on
        // pages that are not CoW.
        let mut pg = user_ptr & !0xFFF;
        let last = (user_ptr + (len as u64) - 1) & !0xFFF;
        loop {
            crate::mm::cow_break(pg);
            if pg >= last {
                break;
            }
            pg += 4096;
        }
        core::ptr::copy_nonoverlapping(src.as_ptr(), user_ptr as *mut u8, len);
    }
    Ok(())
}

pub(crate) unsafe fn copyinstr_user(user_ptr: u64, max: usize) -> Result<Vec<u8>, ()> {
    let limit = if max > USER_COPYINSTR_MAX {
        USER_COPYINSTR_MAX
    } else {
        max
    };
    if !user_range_ok(user_ptr, limit) {
        return Err(());
    }
    if !user_pages_ok(user_ptr, limit, USER_PERM_READ) {
        return Err(());
    }
    let mut out = Vec::new();
    for i in 0..limit {
        let b = *(user_ptr as *const u8).add(i);
        out.push(b)?;
        if b == 0 {
            return Ok(out);
        }
    }
    Err(())
}
