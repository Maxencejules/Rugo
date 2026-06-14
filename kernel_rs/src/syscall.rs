// Syscall entry dispatch and low-level baseline syscalls.

use crate::*;

cfg_m3! {
    static mut MONOTONIC_TICK: u64 = 1;
}

pub(crate) unsafe fn syscall_dispatch(frame: *mut u64) {
    let nr = *frame.add(14);
    let arg1 = *frame.add(9);
    let arg2 = *frame.add(10);
    let arg3 = *frame.add(11);

    #[cfg(feature = "blk_test")]
    {
        if nr == 98 {
            qemu_exit(arg1 as u8);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        match nr {
            0 => {
                *frame.add(14) = sys_debug_write(arg1, arg2);
            }
            3 => {
                *frame.add(14) = sys_yield();
            }
            13 => {
                *frame.add(14) = sys_blk_read(arg1, arg2, arg3);
            }
            14 => {
                *frame.add(14) = sys_blk_write(arg1, arg2, arg3);
            }
            _ => {
                *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            }
        }
        return;
    }

    #[cfg(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test", feature = "go_test"))]
    {
        if nr == 98 {
            qemu_exit(arg1 as u8);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        // Sandbox allowlist (full-os guide Part IV.10): a task that has
        // narrowed its mask via sys_sandbox is denied any syscall whose bit
        // is clear. Default mask is all-ones, so unsandboxed tasks are
        // unaffected.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if nr < 64 && (R4_TASKS[R4_CURRENT].sec_filter_mask >> nr) & 1 == 0 {
                serial_write(b"SANDBOX: deny nr=0x");
                serial_write_hex(nr);
                serial_write(b"\n");
                *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
                return;
            }
        }
        match nr {
            0 => {
                *frame.add(14) = sys_debug_write(arg1, arg2);
            }
            1 => {
                *frame.add(14) = sys_thread_spawn_r4(arg1);
            }
            2 => {
                r4_exit_and_switch(frame, 0);
            }
            3 => {
                r4_yield_and_switch(frame);
            }
            17 => {
                *frame.add(14) = sys_ipc_endpoint_create_r4();
            }
            6 => {
                *frame.add(14) = sys_shm_create_r4(arg1);
            }
            7 => {
                *frame.add(14) = sys_shm_map_r4(arg1, arg2, arg3);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            15 => {
                *frame.add(14) = net::sys_net_send(arg1, arg2);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            16 => {
                *frame.add(14) = net::sys_net_recv(arg1, arg2);
            }
            42 => {
                *frame.add(14) = sys_shm_unmap_r4(arg1);
            }
            8 => {
                *frame.add(14) = sys_ipc_send_r4(arg1, arg2, arg3);
            }
            9 => {
                sys_ipc_recv_r4(frame, arg1, arg2, arg3);
            }
            10 => {
                *frame.add(14) = sys_time_now();
            }
            11 => {
                *frame.add(14) = sys_svc_register_r4(arg1, arg2, arg3);
            }
            12 => {
                *frame.add(14) = sys_svc_lookup_r4(arg1, arg2);
            }
            #[cfg(feature = "go_test")]
            18 => {
                *frame.add(14) = sys_open_v1(arg1, arg2, arg3);
            }
            #[cfg(feature = "go_test")]
            19 => {
                *frame.add(14) = sys_read_v1(arg1, arg2, arg3);
            }
            #[cfg(feature = "go_test")]
            20 => {
                *frame.add(14) = sys_write_v1(arg1, arg2, arg3);
            }
            #[cfg(feature = "go_test")]
            21 => {
                *frame.add(14) = sys_close_v1(arg1);
            }
            22 => {
                sys_wait_r4(frame, arg1, arg2, arg3);
            }
            #[cfg(feature = "go_test")]
            23 => {
                *frame.add(14) = sys_poll_v1(arg1, arg2, arg3);
            }
            28 => {
                *frame.add(14) = sys_proc_info_r4(arg1, arg2, arg3);
            }
            29 => {
                *frame.add(14) = sys_sched_set_r4(arg1, arg2);
            }
            #[cfg(feature = "go_test")]
            30 => {
                *frame.add(14) = sys_fsync_v1(arg1);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            31 => {
                *frame.add(14) = net::sys_socket_open_r4(arg1, arg2);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            32 => {
                *frame.add(14) = net::sys_socket_bind_r4(arg1, arg2, arg3);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            33 => {
                *frame.add(14) = net::sys_socket_listen_r4(arg1, arg2);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            34 => {
                *frame.add(14) = net::sys_socket_connect_r4(arg1, arg2, arg3);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            35 => {
                *frame.add(14) = net::sys_socket_accept_r4(arg1, arg2, arg3);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            36 => {
                *frame.add(14) = net::sys_socket_send_r4(arg1, arg2, arg3);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            37 => {
                *frame.add(14) = net::sys_socket_recv_r4(arg1, arg2, arg3);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            38 => {
                *frame.add(14) = net::sys_socket_close_r4(arg1);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            39 => {
                *frame.add(14) = net::sys_net_if_config_r4(arg1, arg2, arg3);
            }
            #[cfg(any(feature = "net_test", feature = "go_test"))]
            40 => {
                *frame.add(14) = net::sys_net_route_add_r4(arg1, arg2, arg3);
            }
            41 => {
                *frame.add(14) = sys_isolation_config_r4(arg1, arg2, arg3);
            }
            43 => {
                *frame.add(14) = sys_fork_deferred_v1();
            }
            44 => {
                *frame.add(14) = sys_clone_deferred_v1();
            }
            45 => {
                *frame.add(14) = sys_epoll_deferred_v1();
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            46 => {
                let arg4 = *frame.add(5); // r10
                let arg5 = *frame.add(7); // r8
                let arg6 = *frame.add(6); // r9
                *frame.add(14) = sys_spawn_v1(arg1, arg2, arg3, arg4, arg5, arg6);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            47 => {
                *frame.add(14) = sys_fs_ctl_v1(arg1, arg2, arg3);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            48 => {
                sys_signal_ctl(frame, arg1, arg2, arg3);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            49 => {
                *frame.add(14) = sys_net_query(arg1, arg2, arg3);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            50 => {
                let arg4 = *frame.add(5); // r10
                *frame.add(14) = sys_vm_ctl(arg1, arg2, arg3, arg4);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            51 => {
                sys_proc_ctl(frame, arg1, arg2, arg3);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            53 => {
                *frame.add(14) = sys_time(arg1, arg2);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            54 => {
                *frame.add(14) = sys_getrandom(arg1, arg2);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            59 => {
                *frame.add(14) = sys_sandbox(arg1);
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            61 => {
                *frame.add(14) = sys_sysinfo(arg1);
            }
            _ => {
                *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            }
        }
        return;
    }

    #[cfg(not(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test", feature = "go_test")))]
    {
        #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
        {
            if !m10_syscall_allowed(nr) {
                *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
                return;
            }
        }

        if nr == 2 {
            sys_thread_exit_m3(frame);
            return;
        }

        #[cfg(any(feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
        {
            if nr == 98 {
                qemu_exit(arg1 as u8);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
        }

        let ret: u64 = match nr {
            0 => sys_debug_write(arg1, arg2),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            1 => sys_thread_spawn_m3(frame, arg1),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            3 => sys_yield_m3(frame),
            #[cfg(not(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test")))]
            3 => sys_yield(),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            4 => sys_vm_map_m3(arg1, arg2),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            5 => sys_vm_unmap_m3(arg1, arg2),
            10 => sys_time_now(),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            18 => sys_open_v1(arg1, arg2, arg3),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            19 => sys_read_v1(arg1, arg2, arg3),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            20 => sys_write_v1(arg1, arg2, arg3),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            21 => sys_close_v1(arg1),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            22 => sys_wait_v1(arg1, arg2, arg3),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            23 => sys_poll_v1(arg1, arg2, arg3),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            24 => sys_fd_rights_get_v1(arg1),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            25 => sys_fd_rights_reduce_v1(arg1, arg2),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            26 => sys_fd_rights_transfer_v1(arg1, arg2),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            27 => sys_sec_profile_set_v1(arg1),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            30 => sys_fsync_v1(arg1),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            43 => sys_fork_deferred_v1(),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            44 => sys_clone_deferred_v1(),
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            45 => sys_epoll_deferred_v1(),
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        };
        *frame.add(14) = ret;
    }
}

unsafe fn sys_debug_write(buf: u64, len: u64) -> u64 {
    let max_len = 256u64;
    if len > max_len {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let n = len as usize;
    let mut kbuf = [0u8; 256];
    if copyin_user(&mut kbuf, buf, n).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    serial_write(&kbuf[..n]);
    len
}

unsafe fn sys_time_now() -> u64 {
    #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "stress_syscall_test", feature = "user_fault_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
    {
        let t = MONOTONIC_TICK;
        MONOTONIC_TICK += 1;
        return t;
    }
    #[cfg(not(any(feature = "user_hello_test", feature = "syscall_test", feature = "stress_syscall_test", feature = "user_fault_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test")))]
    {
        0
    }
}

unsafe fn sys_yield() -> u64 {
    #[cfg(feature = "sched_test")]
    {
        crate::sched::yield_now();
    }
    0
}
