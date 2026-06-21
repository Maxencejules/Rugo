//! `sys_epoll` (ABI v3.x id 55): a level-triggered readiness set over the fd/pipe
//! tables — "stateful poll". Register a set of fds once, then wait repeatedly.
//!
//! Extracted from `lib.rs` (gap #9, maintainability). This module owns the epoll
//! *instance* state and op dispatch; the per-fd readiness rules stay with the fd
//! tables in `lib.rs` (`crate::epoll_fd_ready`), which is the natural owner of the
//! fd/pipe state it reads.

use crate::memory::{copyout_user, user_pages_ok, user_range_ok, USER_PERM_READ, USER_PERM_WRITE};

const EPOLL_MAX: usize = 4;
const EPOLL_REG_MAX: usize = 16;

#[derive(Clone, Copy)]
struct EpollReg {
    fd: i32,
    events: u16,
}

struct EpollInst {
    active: bool,
    n: usize,
    regs: [EpollReg; EPOLL_REG_MAX],
}

static mut EPOLLS: [EpollInst; EPOLL_MAX] = [
    EpollInst { active: false, n: 0, regs: [EpollReg { fd: 0, events: 0 }; EPOLL_REG_MAX] },
    EpollInst { active: false, n: 0, regs: [EpollReg { fd: 0, events: 0 }; EPOLL_REG_MAX] },
    EpollInst { active: false, n: 0, regs: [EpollReg { fd: 0, events: 0 }; EPOLL_REG_MAX] },
    EpollInst { active: false, n: 0, regs: [EpollReg { fd: 0, events: 0 }; EPOLL_REG_MAX] },
];

/// op 1 = create (returns an epoll instance id), op 2 = ctl_add(ep, fd, events),
/// op 3 = wait(ep, out_ptr, max) writing ready `{fd:i32, revents:u16, pad:u16}`
/// 8-byte records and returning the count, op 4 = close(ep). EPOLLIN/EPOLLOUT use
/// the same 0x1/0x4 bits as poll.
pub(crate) unsafe fn sys_epoll(op: u64, a2: u64, a3: u64, a4: u64) -> u64 {
    const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    match op {
        1 => {
            // create: claim a free instance
            let mut e = 0usize;
            while e < EPOLL_MAX {
                if !EPOLLS[e].active {
                    EPOLLS[e].active = true;
                    EPOLLS[e].n = 0;
                    return e as u64;
                }
                e += 1;
            }
            ERR
        }
        2 => {
            // ctl_add(ep, fd, events)
            let ep = a2 as usize;
            if ep >= EPOLL_MAX || !EPOLLS[ep].active || EPOLLS[ep].n >= EPOLL_REG_MAX {
                return ERR;
            }
            let n = EPOLLS[ep].n;
            EPOLLS[ep].regs[n] = EpollReg { fd: a3 as i32, events: a4 as u16 };
            EPOLLS[ep].n += 1;
            0
        }
        3 => {
            // wait(ep, out_ptr, max) -> ready count; writes 8-byte {fd,revents,pad}
            let ep = a2 as usize;
            let out_ptr = a3;
            let max = a4 as usize;
            if ep >= EPOLL_MAX || !EPOLLS[ep].active || max == 0 {
                return ERR;
            }
            let total = match max.checked_mul(8) {
                Some(v) => v,
                None => return ERR,
            };
            if !user_range_ok(out_ptr, total)
                || !user_pages_ok(out_ptr, total, USER_PERM_READ | USER_PERM_WRITE)
            {
                return ERR;
            }
            let mut count = 0usize;
            let mut i = 0usize;
            while i < EPOLLS[ep].n && count < max {
                let reg = EPOLLS[ep].regs[i];
                let re = crate::epoll_fd_ready(reg.fd, reg.events);
                if re != 0 {
                    let mut rec = [0u8; 8];
                    rec[0..4].copy_from_slice(&reg.fd.to_le_bytes());
                    rec[4..6].copy_from_slice(&re.to_le_bytes());
                    if copyout_user(out_ptr + (count * 8) as u64, &rec, 8).is_err() {
                        return ERR;
                    }
                    count += 1;
                }
                i += 1;
            }
            count as u64
        }
        4 => {
            // close
            let ep = a2 as usize;
            if ep < EPOLL_MAX {
                EPOLLS[ep].active = false;
                EPOLLS[ep].n = 0;
            }
            0
        }
        _ => ERR,
    }
}
