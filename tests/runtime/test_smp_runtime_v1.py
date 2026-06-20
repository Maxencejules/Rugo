# Phase 10d acceptance: SMP bring-up groundwork. Booted with -smp 4,
# the kernel must report all four CPUs from the Limine SMP response and
# every application processor must check in (run real kernel code on
# its own core) before parking. The default Go lane must also boot and
# shut down cleanly on multicore with the APs parked.

import os
import socket
import subprocess
import sys
import time
import uuid

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import conftest  # noqa: E402


def _boot_smp(iso, smp, input_text=None, with_devices=False, timeout=30):
    serial_port = conftest._pick_serial_port()
    cmd = [
        conftest.QEMU_BIN,
        "-machine", "q35", "-cpu", "qemu64,+x2apic", "-smp", str(smp), "-m", "256",
        "-display", "none", "-no-reboot",
        "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
        "-cdrom", iso,
        "-serial", f"tcp:127.0.0.1:{serial_port},server=on,wait=off",
    ]
    disk = None
    if with_devices:
        disk = os.path.join(conftest.REPO_ROOT, "out", f"smp-{uuid.uuid4().hex}.img")
        conftest._ensure_app_region(disk)
        cmd += [
            "-drive", f"file={disk},if=none,id=disk0,format=raw",
            "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
            "-netdev", "user,id=n0",
            "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        ]
    proc = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    transcript = ""
    try:
        serial = conftest._connect_serial(serial_port, proc, 20)
        deadline = time.monotonic() + timeout
        sent = input_text is None
        while time.monotonic() < deadline and proc.poll() is None:
            try:
                chunk = serial.recv(4096)
            except socket.timeout:
                chunk = None
            except OSError:
                break
            if chunk:
                transcript += chunk.decode("utf-8", errors="replace")
            if not sent and "GOSH: session ready" in transcript:
                serial.sendall(input_text.encode())
                sent = True
        try:
            while True:
                chunk = serial.recv(4096)
                if not chunk:
                    break
                transcript += chunk.decode("utf-8", errors="replace")
        except OSError:
            pass
        serial.close()
    finally:
        if proc.poll() is None:
            proc.kill()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            pass
        if disk:
            for _ in range(20):
                try:
                    if os.path.isfile(disk):
                        os.remove(disk)
                    break
                except PermissionError:
                    time.sleep(0.25)
    return transcript


def test_aps_check_in_on_quad_core(find_in_order):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    out = _boot_smp(iso, 4)
    find_in_order(out, [
        "RUGO: boot ok",
        "SMP: cpus=0x0000000000000004",
        "SMP: aps online=0x0000000000000003",
        # Spinlock contention: 4 CPUs x 2000 locked increments = 8000 = 0x1F40,
        # with no lost updates -> the lock serialized every core.
        "SMP: lock count=0x0000000000001F40 ok",
        # IPI: the BSP broadcasts to all 3 APs, each takes vector 240 and acks.
        "SMP: ipi ack=0x0000000000000003",
        # Per-CPU LAPIC timers: every AP's own preemption clock fired.
        "SMP: ap timers ok",
        # TLB shootdown: the BSP directed all 3 APs to invalidate an address
        # and every one acknowledged (cross-CPU TLB invalidation works).
        "SMP: tlb shootdown ok",
        # Per-CPU storage: each AP recorded its index through its own GS base.
        "SMP: percpu ok",
        # Cross-CPU work dispatch: an AP claimed + ran a dispatched computation.
        "SMP: ap work ok",
        # Per-CPU run queues: each AP drained its own queue to the right total.
        "SMP: runqueue ok",
        # Per-CPU affinity: the BSP routed DISTINCT work to each core and each ran
        # only its own (the load-balancing basis), with the batch fully distributed.
        "SMP: affinity ok",
        "RUGO: halt ok",
    ])
    assert "SMP: lock count" in out
    assert " FAIL" not in out


def test_default_lane_boots_clean_on_multicore(find_in_order):
    iso = os.path.join(conftest.REPO_ROOT, "out", "os-go.iso")
    if not os.path.isfile(iso):
        import pytest

        pytest.skip(f"ISO not built: {iso}")
    out = _boot_smp(iso, 2, input_text="shutdown\n", with_devices=True, timeout=40)
    find_in_order(out, [
        "SMP: cpus=0x0000000000000002",
        "SMP: aps online=0x0000000000000001",
        # 2 CPUs x 2000 locked increments = 4000 = 0xFA0, no lost updates.
        "SMP: lock count=0x0000000000000FA0 ok",
        # IPI: the BSP signals the single AP, which takes vector 240 and acks.
        "SMP: ipi ack=0x0000000000000001",
        # The AP's own LAPIC timer fired.
        "SMP: ap timers ok",
        # TLB shootdown: the BSP directed the single AP to invalidate and it acked.
        "SMP: tlb shootdown ok",
        # Per-CPU storage: the AP recorded its index through its own GS base.
        "SMP: percpu ok",
        # Cross-CPU work dispatch: the AP claimed + ran a dispatched computation.
        "SMP: ap work ok",
        # Per-CPU run queues: the AP drained its own queue to the right total.
        "SMP: runqueue ok",
        # Affinity invariant (live per-CPU scheduler): with AP-eligible tasks
        # planted INSIDE the BSP's live rotation [1,R4_NUM_TASKS), the BSP's
        # r4_find_ready only ever returns the non-eligible task and is starved
        # when the whole window is AP-eligible -- proof the BSP and the APs run
        # disjoint sets of the same task table, so no task runs on two CPUs.
        "SMP: affinity live-skip ok",
        # Capstone: a real ring-3 USER task ran on the application processor
        # (not the BSP). The AP entered ring 3 on its own per-CPU TSS rsp0, set
        # its per-CPU `current` task through GS, serviced TWO real syscalls
        # (int 0x80 sys_time_now) whose tick delta is exactly 1 -- proof real
        # kernel code ran for each on the AP's own core -- then reported arg*2+1
        # via int 0x81 and returned to the kernel cleanly.
        "SMP: ap-syscall delta=0x0000000000000001",
        "SMP: ap-current=0x000000000000005A",
        "SMP: ap user task ok",
        # A REAL R4 task (an R4_TASKS scheduler entry created via r4_init_task) was
        # migrated to the AP: the AP ran its CR3 + ring-3 context, tracked its real
        # tid (slot 0x1F) as its per-CPU `current`, and serviced its syscalls. Two of
        # its own real syscalls resolved its own slot through per-CPU state on the AP:
        # a READ (getuid -> R4_TASKS[r4_current_smp()].uid == 0x77, the BSP's sentinel)
        # and a WRITE (op 16 bumped this slot's yield_count -> scyc=0x1). The per-CPU
        # R4_CURRENT reroute working both directions on the AP, indexing the real table.
        "SMP: ap r4 migrate tid=0x000000000000001F cur=0x000000000000001F scuid=0x0000000000000077 scyc=0x0000000000000001 ok",
        # Concurrency: a ring-3 task on the AP and the BSP completed a rendezvous
        # (the AP signalled arrival + waited in-kernel for the BSP's ack while the
        # BSP, having dispatched ASYNCHRONOUSLY, supplied it). This can only close if
        # both CPUs run at the same instant -> two tasks on two CPUs at once.
        "SMP: ap+bsp concurrent rv=0x00000000000000AC ok",
        # Live scheduler with PER-CPU AFFINITY: the 3 tasks are flagged ap_eligible,
        # so the BSP's r4_find_ready SKIPS them and an AP claims them by scanning the
        # live task table for ap_eligible Ready tasks (under the run-queue lock) and
        # runs them in ring 3 -- no per-task dispatch from the BSP, each exactly once.
        "SMP: live sched ran=0x0000000000000003 ap-affinity ok",
        # Preemptible AP task (live per-CPU scheduler slice 4): an AP runs a
        # CPU-bound task in ring 3 with IF=1, and its OWN periodic LAPIC timer
        # fires INSIDE the running task (counted in the hits= field, >0) -- proof
        # the AP's preemption clock lands mid-task, the precondition for
        # time-slicing on an application processor. The task's loop counter
        # survives each interrupt in its saved frame, so it still completes (ran=1).
        "SMP: ap preempt ok",
        # sys_sysinfo op 13 reports the online CPU count (BSP + 1 AP = 2) via the
        # real syscall dispatch path, sized from the live SMP state.
        "CPUS: count=0x0000000000000002",
        "GOSH: session ready",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "USERPF" not in out
    assert "GOINIT: err" not in out
    assert " FAIL" not in out
