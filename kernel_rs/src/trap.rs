// Trap entry and user-fault containment.

use crate::arch_x86::qemu_exit;
use crate::runtime;
use crate::{serial_write, serial_write_hex, stack_top};

#[no_mangle]
pub extern "C" fn trap_handler(frame: *mut u64) {
    unsafe {
        let int_num = *frame.add(15);
        let error_code = *frame.add(16);

        match int_num {
            0 => {
                serial_write(b"TRAP: div0\n");
                qemu_exit(0x31);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
            3 => {
                serial_write(b"TRAP: ok\n");
                qemu_exit(0x31);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
            8 => {
                serial_write(b"TRAP: double fault\n");
                qemu_exit(0x31);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
            13 => {
                let cs = *frame.add(18);
                if cs & 3 == 3 {
                    #[cfg(feature = "go_test")]
                    {
                        serial_write(b"USERGPF: err=0x");
                        serial_write_hex(error_code);
                        serial_write(b" rip=0x");
                        serial_write_hex(*frame.add(17));
                        serial_write(b"\n");
                    }
                    handle_user_fault(frame);
                    return;
                }
                serial_write(b"TRAP: gpf err=0x");
                serial_write_hex(error_code);
                serial_write(b"\n");
                qemu_exit(0x31);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
            14 => {
                let cs = *frame.add(18);
                if cs & 3 == 3 {
                    let demand_cr2: u64;
                    core::arch::asm!("mov {}, cr2", out(reg) demand_cr2, options(nomem, nostack));
                    if crate::mm::try_demand_map(demand_cr2) {
                        return;
                    }
                    #[cfg(feature = "go_test")]
                    {
                        let cr2: u64;
                        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack));
                        serial_write(b"USERPF: addr=0x");
                        serial_write_hex(cr2);
                        serial_write(b" err=0x");
                        serial_write_hex(error_code);
                        serial_write(b" rip=0x");
                        serial_write_hex(*frame.add(17));
                        serial_write(b"\n");
                    }
                    handle_user_fault(frame);
                    return;
                }
                let cr2: u64;
                core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack));
                serial_write(b"PF: addr=0x");
                serial_write_hex(cr2);
                serial_write(b" err=0x");
                serial_write_hex(error_code);
                serial_write(b"\n");
                qemu_exit(0x31);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
            #[cfg(feature = "sched_test")]
            32 => {
                crate::sched::handle_timer_irq();
            }
            #[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
            64 | 65 => {
                if runtime::native::handle_irq(int_num) {
                    return;
                }
            }
            128 => {
                crate::syscall::syscall_dispatch(frame);
            }
            _ => {}
        }
    }
}

extern "C" fn user_fault_return() -> ! {
    serial_write(b"RUGO: halt ok\n");
    qemu_exit(0x31);
    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
    }
}

#[cfg(not(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test")))]
pub(crate) unsafe fn m3_return_to_kernel_halt(frame: *mut u64) {
    let kstack = &stack_top as *const u8 as u64;
    *frame.add(17) = user_fault_return as *const () as u64;
    *frame.add(18) = 0x08;
    *frame.add(19) = 0x02;
    *frame.add(20) = kstack;
    *frame.add(21) = 0x10;
}

unsafe fn handle_user_fault(frame: *mut u64) {
    #[cfg(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test", feature = "go_test"))]
    {
        crate::r4_exit_and_switch(frame, 1);
        return;
    }

    #[cfg(not(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test", feature = "go_test")))]
    {
        serial_write(b"USER: killed\n");
        m3_return_to_kernel_halt(frame);
    }
}
