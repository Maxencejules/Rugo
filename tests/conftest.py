import os
import shutil
import socket
import subprocess
import sys
import threading
import time

import pytest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ISO_PATH = os.path.join(REPO_ROOT, "out", "os.iso")
ISO_PANIC_PATH = os.path.join(REPO_ROOT, "out", "os-panic.iso")
ISO_PF_PATH = os.path.join(REPO_ROOT, "out", "os-pf.iso")
ISO_IDT_PATH = os.path.join(REPO_ROOT, "out", "os-idt.iso")
ISO_SCHED_PATH = os.path.join(REPO_ROOT, "out", "os-sched.iso")
ISO_USER_HELLO_PATH = os.path.join(REPO_ROOT, "out", "os-user-hello.iso")
ISO_SYSCALL_PATH = os.path.join(REPO_ROOT, "out", "os-syscall.iso")
ISO_THREAD_EXIT_PATH = os.path.join(REPO_ROOT, "out", "os-thread-exit.iso")
ISO_THREAD_SPAWN_PATH = os.path.join(REPO_ROOT, "out", "os-thread-spawn.iso")
ISO_VM_MAP_PATH = os.path.join(REPO_ROOT, "out", "os-vm-map.iso")
ISO_SYSCALL_INVALID_PATH = os.path.join(REPO_ROOT, "out", "os-syscall-invalid.iso")
ISO_STRESS_SYSCALL_PATH = os.path.join(REPO_ROOT, "out", "os-stress-syscall.iso")
ISO_STRESS_IPC_PATH = os.path.join(REPO_ROOT, "out", "os-stress-ipc.iso")
ISO_YIELD_PATH = os.path.join(REPO_ROOT, "out", "os-yield.iso")
ISO_USER_FAULT_PATH = os.path.join(REPO_ROOT, "out", "os-user-fault.iso")
ISO_IPC_PATH = os.path.join(REPO_ROOT, "out", "os-ipc.iso")
ISO_IPC_BADPTR_SEND_PATH = os.path.join(REPO_ROOT, "out", "os-ipc-badptr-send.iso")
ISO_IPC_BADPTR_RECV_PATH = os.path.join(REPO_ROOT, "out", "os-ipc-badptr-recv.iso")
ISO_SVC_BADPTR_PATH = os.path.join(REPO_ROOT, "out", "os-svc-badptr.iso")
# Backward-compatible alias
ISO_IPC_BADPTR_SVC_PATH = ISO_SVC_BADPTR_PATH
ISO_IPC_BUFFER_FULL_PATH = os.path.join(REPO_ROOT, "out", "os-ipc-buffer-full.iso")
ISO_IPC_WAITER_BUSY_PATH = os.path.join(REPO_ROOT, "out", "os-ipc-waiter-busy.iso")
ISO_SVC_OVERWRITE_PATH = os.path.join(REPO_ROOT, "out", "os-svc-overwrite.iso")
# Backward-compatible alias
ISO_IPC_SVC_OVERWRITE_PATH = ISO_SVC_OVERWRITE_PATH
ISO_SVC_FULL_PATH = os.path.join(REPO_ROOT, "out", "os-svc-full.iso")
ISO_SVC_BAD_ENDPOINT_PATH = os.path.join(REPO_ROOT, "out", "os-svc-bad-endpoint.iso")
ISO_SHM_PATH = os.path.join(REPO_ROOT, "out", "os-shm.iso")
ISO_PRESSURE_SHM_PATH = os.path.join(REPO_ROOT, "out", "os-pressure-shm.iso")
ISO_QUOTA_ENDPOINTS_PATH = os.path.join(REPO_ROOT, "out", "os-quota-endpoints.iso")
ISO_QUOTA_SHM_PATH = os.path.join(REPO_ROOT, "out", "os-quota-shm.iso")
ISO_QUOTA_THREADS_PATH = os.path.join(REPO_ROOT, "out", "os-quota-threads.iso")
ISO_BLK_PATH = os.path.join(REPO_ROOT, "out", "os-blk.iso")
ISO_STRESS_BLK_PATH = os.path.join(REPO_ROOT, "out", "os-stress-blk.iso")
ISO_BLK_BADLEN_PATH = os.path.join(REPO_ROOT, "out", "os-blk-badlen.iso")
ISO_BLK_BADPTR_PATH = os.path.join(REPO_ROOT, "out", "os-blk-badptr.iso")
ISO_BLK_INVARIANTS_PATH = os.path.join(REPO_ROOT, "out", "os-blk-invariants.iso")
ISO_BLK_INIT_FAIL_PATH = os.path.join(REPO_ROOT, "out", "os-blk-init-fail.iso")
BLK_DISK_IMG = os.path.join(REPO_ROOT, "out", "blk-test.img")
ISO_FS_PATH = os.path.join(REPO_ROOT, "out", "os-fs.iso")
FS_DISK_IMG = os.path.join(REPO_ROOT, "out", "fs-test.img")
ISO_FS_BADMAGIC_PATH = os.path.join(REPO_ROOT, "out", "os-fs-badmagic.iso")
FS_BADMAGIC_DISK_IMG = os.path.join(REPO_ROOT, "out", "fs-badmagic.img")
ISO_PKG_HASH_PATH = os.path.join(REPO_ROOT, "out", "os-pkg-hash.iso")
FS_EXTERNAL_DISK_IMG = os.path.join(REPO_ROOT, "out", "fs-external.img")
PKG_V1_PATH = os.path.join(REPO_ROOT, "out", "hello.pkgv1")
PKG_REPO_V1_PATH = os.path.join(REPO_ROOT, "out", "repo-v1.json")
PKG_BOOTSTRAP_V1_TOOL = os.path.join(REPO_ROOT, "tools", "pkg_bootstrap_v1.py")
ISO_NET_PATH = os.path.join(REPO_ROOT, "out", "os-net.iso")
ISO_GO_PATH = os.path.join(REPO_ROOT, "out", "os-go.iso")
ISO_GO_STD_PATH = os.path.join(REPO_ROOT, "out", "os-go-std.iso")
QEMU_TIMEOUT = 10  # seconds
NET_TIMEOUT = 15   # longer timeout for networking


def _resolve_qemu_bin():
    """Resolve QEMU binary path across Unix/Windows setups."""
    candidates = [os.environ.get("QEMU_BIN")]

    qemu_in_path = shutil.which("qemu-system-x86_64")
    if qemu_in_path:
        candidates.append(qemu_in_path)

    qemu_exe_in_path = shutil.which("qemu-system-x86_64.exe")
    if qemu_exe_in_path:
        candidates.append(qemu_exe_in_path)

    if os.name == "nt":
        program_files = os.environ.get("ProgramFiles", r"C:\Program Files")
        candidates.append(
            os.path.join(program_files, "qemu", "qemu-system-x86_64.exe")
        )

    for cand in candidates:
        if cand and os.path.isfile(cand):
            return cand
    return None


QEMU_BIN = _resolve_qemu_bin()


def _boot_iso(iso_path):
    """Boot an ISO in QEMU headless and return the CompletedProcess."""
    assert os.path.isfile(iso_path), f"ISO not found: {iso_path}"
    if not QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found (set QEMU_BIN or install QEMU)")

    try:
        result = subprocess.run(
            [
                QEMU_BIN,
                "-machine", "q35",
                "-cpu", "qemu64",
                "-m", "128",
                "-serial", "stdio",
                "-display", "none",
                "-no-reboot",
                "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
                "-cdrom", iso_path,
            ],
            capture_output=True,
            text=True,
            timeout=QEMU_TIMEOUT,
        )
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout.decode("utf-8", errors="replace") if exc.stdout else ""
        pytest.fail(f"QEMU timed out ({QEMU_TIMEOUT}s). Captured serial:\n{stdout}")

    return result


@pytest.fixture
def qemu_serial():
    """Boot the normal OS image and return captured serial output."""
    return _boot_iso(ISO_PATH)


@pytest.fixture
def qemu_serial_panic():
    """Boot the panic-test OS image and return captured serial output."""
    return _boot_iso(ISO_PANIC_PATH)


@pytest.fixture
def qemu_serial_pf():
    """Boot the page-fault-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_PF_PATH):
        pytest.skip(f"ISO not built: {ISO_PF_PATH}")
    return _boot_iso(ISO_PF_PATH)


@pytest.fixture
def qemu_serial_idt():
    """Boot the IDT-smoke-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_IDT_PATH):
        pytest.skip(f"ISO not built: {ISO_IDT_PATH}")
    return _boot_iso(ISO_IDT_PATH)


@pytest.fixture
def qemu_serial_sched():
    """Boot the scheduler-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SCHED_PATH):
        pytest.skip(f"ISO not built: {ISO_SCHED_PATH}")
    return _boot_iso(ISO_SCHED_PATH)


@pytest.fixture
def qemu_serial_user_hello():
    """Boot the user-hello-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_USER_HELLO_PATH):
        pytest.skip(f"ISO not built: {ISO_USER_HELLO_PATH}")
    return _boot_iso(ISO_USER_HELLO_PATH)


@pytest.fixture
def qemu_serial_syscall():
    """Boot the syscall-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SYSCALL_PATH):
        pytest.skip(f"ISO not built: {ISO_SYSCALL_PATH}")
    return _boot_iso(ISO_SYSCALL_PATH)


@pytest.fixture
def qemu_serial_thread_exit():
    """Boot the thread-exit-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_THREAD_EXIT_PATH):
        pytest.skip(f"ISO not built: {ISO_THREAD_EXIT_PATH}")
    return _boot_iso(ISO_THREAD_EXIT_PATH)


@pytest.fixture
def qemu_serial_thread_spawn():
    """Boot the thread-spawn-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_THREAD_SPAWN_PATH):
        pytest.skip(f"ISO not built: {ISO_THREAD_SPAWN_PATH}")
    return _boot_iso(ISO_THREAD_SPAWN_PATH)


@pytest.fixture
def qemu_serial_vm_map():
    """Boot the vm-map-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_VM_MAP_PATH):
        pytest.skip(f"ISO not built: {ISO_VM_MAP_PATH}")
    return _boot_iso(ISO_VM_MAP_PATH)


@pytest.fixture
def _boot_iso_syscall_invalid():
    """Boot the invalid-syscall-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SYSCALL_INVALID_PATH):
        pytest.skip(f"ISO not built: {ISO_SYSCALL_INVALID_PATH}")
    return _boot_iso(ISO_SYSCALL_INVALID_PATH)


@pytest.fixture
def qemu_serial_stress_syscall():
    """Boot the stress-syscall-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_STRESS_SYSCALL_PATH):
        pytest.skip(f"ISO not built: {ISO_STRESS_SYSCALL_PATH}")
    return _boot_iso(ISO_STRESS_SYSCALL_PATH)


@pytest.fixture
def qemu_serial_stress_ipc():
    """Boot the stress-ipc-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_STRESS_IPC_PATH):
        pytest.skip(f"ISO not built: {ISO_STRESS_IPC_PATH}")
    return _boot_iso(ISO_STRESS_IPC_PATH)


@pytest.fixture
def qemu_serial_yield():
    """Boot the yield-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_YIELD_PATH):
        pytest.skip(f"ISO not built: {ISO_YIELD_PATH}")
    return _boot_iso(ISO_YIELD_PATH)


@pytest.fixture
def qemu_serial_user_fault():
    """Boot the user-fault-test OS image and return captured serial output."""
    if not os.path.isfile(ISO_USER_FAULT_PATH):
        pytest.skip(f"ISO not built: {ISO_USER_FAULT_PATH}")
    return _boot_iso(ISO_USER_FAULT_PATH)


@pytest.fixture
def qemu_serial_ipc():
    """Boot the IPC ping-pong test OS image and return captured serial output."""
    if not os.path.isfile(ISO_IPC_PATH):
        pytest.skip(f"ISO not built: {ISO_IPC_PATH}")
    return _boot_iso(ISO_IPC_PATH)


@pytest.fixture
def qemu_serial_ipc_badptr_send():
    """Boot the IPC bad-pointer send test OS image and return captured serial output."""
    if not os.path.isfile(ISO_IPC_BADPTR_SEND_PATH):
        pytest.skip(f"ISO not built: {ISO_IPC_BADPTR_SEND_PATH}")
    return _boot_iso(ISO_IPC_BADPTR_SEND_PATH)


@pytest.fixture
def qemu_serial_ipc_badptr_recv():
    """Boot the IPC bad-pointer recv test OS image and return captured serial output."""
    if not os.path.isfile(ISO_IPC_BADPTR_RECV_PATH):
        pytest.skip(f"ISO not built: {ISO_IPC_BADPTR_RECV_PATH}")
    return _boot_iso(ISO_IPC_BADPTR_RECV_PATH)


@pytest.fixture
def qemu_serial_svc_badptr():
    """Boot the service-registry bad-pointer test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SVC_BADPTR_PATH):
        pytest.skip(f"ISO not built: {ISO_SVC_BADPTR_PATH}")
    return _boot_iso(ISO_SVC_BADPTR_PATH)


@pytest.fixture
def qemu_serial_ipc_badptr_svc(qemu_serial_svc_badptr):
    """Backward-compatible alias for the service-registry bad-pointer fixture."""
    return qemu_serial_svc_badptr


@pytest.fixture
def qemu_serial_ipc_buffer_full():
    """Boot the IPC buffer-full test OS image and return captured serial output."""
    if not os.path.isfile(ISO_IPC_BUFFER_FULL_PATH):
        pytest.skip(f"ISO not built: {ISO_IPC_BUFFER_FULL_PATH}")
    return _boot_iso(ISO_IPC_BUFFER_FULL_PATH)


@pytest.fixture
def qemu_serial_ipc_waiter_busy():
    """Boot the IPC waiter-busy semantics test OS image."""
    if not os.path.isfile(ISO_IPC_WAITER_BUSY_PATH):
        pytest.skip(f"ISO not built: {ISO_IPC_WAITER_BUSY_PATH}")
    return _boot_iso(ISO_IPC_WAITER_BUSY_PATH)


@pytest.fixture
def qemu_serial_svc_overwrite():
    """Boot the SVC overwrite test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SVC_OVERWRITE_PATH):
        pytest.skip(f"ISO not built: {ISO_SVC_OVERWRITE_PATH}")
    return _boot_iso(ISO_SVC_OVERWRITE_PATH)


@pytest.fixture
def qemu_serial_ipc_svc_overwrite(qemu_serial_svc_overwrite):
    """Backward-compatible alias for the SVC overwrite fixture."""
    return qemu_serial_svc_overwrite


@pytest.fixture
def qemu_serial_svc_full():
    """Boot the SVC table-full test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SVC_FULL_PATH):
        pytest.skip(f"ISO not built: {ISO_SVC_FULL_PATH}")
    return _boot_iso(ISO_SVC_FULL_PATH)


@pytest.fixture
def qemu_serial_svc_bad_endpoint():
    """Boot the SVC bad-endpoint test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SVC_BAD_ENDPOINT_PATH):
        pytest.skip(f"ISO not built: {ISO_SVC_BAD_ENDPOINT_PATH}")
    return _boot_iso(ISO_SVC_BAD_ENDPOINT_PATH)


@pytest.fixture
def qemu_serial_shm():
    """Boot the SHM bulk test OS image and return captured serial output."""
    if not os.path.isfile(ISO_SHM_PATH):
        pytest.skip(f"ISO not built: {ISO_SHM_PATH}")
    return _boot_iso(ISO_SHM_PATH)


@pytest.fixture
def qemu_serial_pressure_shm():
    """Boot the SHM pressure test OS image and return captured serial output."""
    if not os.path.isfile(ISO_PRESSURE_SHM_PATH):
        pytest.skip(f"ISO not built: {ISO_PRESSURE_SHM_PATH}")
    return _boot_iso(ISO_PRESSURE_SHM_PATH)


@pytest.fixture
def qemu_serial_quota_endpoints():
    """Boot the endpoint quota test OS image and return captured serial output."""
    if not os.path.isfile(ISO_QUOTA_ENDPOINTS_PATH):
        pytest.skip(f"ISO not built: {ISO_QUOTA_ENDPOINTS_PATH}")
    return _boot_iso(ISO_QUOTA_ENDPOINTS_PATH)


@pytest.fixture
def qemu_serial_quota_shm():
    """Boot the SHM quota test OS image and return captured serial output."""
    if not os.path.isfile(ISO_QUOTA_SHM_PATH):
        pytest.skip(f"ISO not built: {ISO_QUOTA_SHM_PATH}")
    return _boot_iso(ISO_QUOTA_SHM_PATH)


@pytest.fixture
def qemu_serial_quota_threads():
    """Boot the thread quota test OS image and return captured serial output."""
    if not os.path.isfile(ISO_QUOTA_THREADS_PATH):
        pytest.skip(f"ISO not built: {ISO_QUOTA_THREADS_PATH}")
    return _boot_iso(ISO_QUOTA_THREADS_PATH)


def _boot_iso_with_disk(iso_path, disk_path):
    """Boot an ISO in QEMU with a virtio-blk disk attached."""
    assert os.path.isfile(iso_path), f"ISO not found: {iso_path}"
    if not QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found (set QEMU_BIN or install QEMU)")

    # Create a 1 MiB raw disk image if it doesn't exist
    if not os.path.isfile(disk_path):
        with open(disk_path, "wb") as f:
            f.write(b"\x00" * (1024 * 1024))

    try:
        result = subprocess.run(
            [
                QEMU_BIN,
                "-machine", "q35",
                "-cpu", "qemu64",
                "-m", "128",
                "-serial", "stdio",
                "-display", "none",
                "-no-reboot",
                "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
                "-cdrom", iso_path,
                "-drive", f"file={disk_path},format=raw,if=none,id=disk0",
                "-device", "virtio-blk-pci,drive=disk0,disable-modern=on",
            ],
            capture_output=True,
            text=True,
            timeout=QEMU_TIMEOUT,
        )
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout.decode("utf-8", errors="replace") if exc.stdout else ""
        pytest.fail(f"QEMU timed out ({QEMU_TIMEOUT}s). Captured serial:\n{stdout}")

    return result


@pytest.fixture
def qemu_serial_blk():
    """Boot the VirtIO block test OS image with a raw disk attached."""
    if not os.path.isfile(ISO_BLK_PATH):
        pytest.skip(f"ISO not built: {ISO_BLK_PATH}")
    return _boot_iso_with_disk(ISO_BLK_PATH, BLK_DISK_IMG)


@pytest.fixture
def qemu_serial_stress_blk():
    """Boot the stress VirtIO block test OS image with a raw disk attached."""
    if not os.path.isfile(ISO_STRESS_BLK_PATH):
        pytest.skip(f"ISO not built: {ISO_STRESS_BLK_PATH}")
    return _boot_iso_with_disk(ISO_STRESS_BLK_PATH, BLK_DISK_IMG)


@pytest.fixture
def qemu_serial_blk_badlen():
    """Boot the VirtIO block bad-length test OS image with a raw disk."""
    if not os.path.isfile(ISO_BLK_BADLEN_PATH):
        pytest.skip(f"ISO not built: {ISO_BLK_BADLEN_PATH}")
    return _boot_iso_with_disk(ISO_BLK_BADLEN_PATH, BLK_DISK_IMG)


@pytest.fixture
def qemu_serial_blk_badptr():
    """Boot the VirtIO block bad-pointer test OS image with a raw disk."""
    if not os.path.isfile(ISO_BLK_BADPTR_PATH):
        pytest.skip(f"ISO not built: {ISO_BLK_BADPTR_PATH}")
    return _boot_iso_with_disk(ISO_BLK_BADPTR_PATH, BLK_DISK_IMG)


@pytest.fixture
def qemu_serial_blk_invariants():
    """Boot the VirtIO block init-invariants test OS image with a raw disk."""
    if not os.path.isfile(ISO_BLK_INVARIANTS_PATH):
        pytest.skip(f"ISO not built: {ISO_BLK_INVARIANTS_PATH}")
    return _boot_iso_with_disk(ISO_BLK_INVARIANTS_PATH, BLK_DISK_IMG)


@pytest.fixture
def qemu_serial_blk_init_fail():
    """Boot the VirtIO block init-failure test OS image with a raw disk."""
    if not os.path.isfile(ISO_BLK_INIT_FAIL_PATH):
        pytest.skip(f"ISO not built: {ISO_BLK_INIT_FAIL_PATH}")
    return _boot_iso_with_disk(ISO_BLK_INIT_FAIL_PATH, BLK_DISK_IMG)


@pytest.fixture
def qemu_serial_fs():
    """Boot the FS test OS image with a formatted SimpleFS disk attached."""
    if not os.path.isfile(ISO_FS_PATH):
        pytest.skip(f"ISO not built: {ISO_FS_PATH}")
    if not os.path.isfile(FS_DISK_IMG):
        pytest.skip(f"Disk image not built: {FS_DISK_IMG}")
    return _boot_iso_with_disk(ISO_FS_PATH, FS_DISK_IMG)


@pytest.fixture
def qemu_serial_fs_badmagic():
    """Boot the FS bad-magic test OS image with a corrupted superblock disk."""
    if not os.path.isfile(ISO_FS_BADMAGIC_PATH):
        pytest.skip(f"ISO not built: {ISO_FS_BADMAGIC_PATH}")
    if not os.path.isfile(FS_BADMAGIC_DISK_IMG):
        pytest.skip(f"Disk image not built: {FS_BADMAGIC_DISK_IMG}")
    return _boot_iso_with_disk(ISO_FS_BADMAGIC_PATH, FS_BADMAGIC_DISK_IMG)


@pytest.fixture
def qemu_serial_pkg_hash():
    """Boot the PKG hash test OS image with a formatted SimpleFS disk attached."""
    if not os.path.isfile(ISO_PKG_HASH_PATH):
        pytest.skip(f"ISO not built: {ISO_PKG_HASH_PATH}")
    if not os.path.isfile(FS_DISK_IMG):
        pytest.skip(f"Disk image not built: {FS_DISK_IMG}")
    return _boot_iso_with_disk(ISO_PKG_HASH_PATH, FS_DISK_IMG)


@pytest.fixture
def external_pkg_disk_img():
    """Build an external app package/repo image via the v1 bootstrap tool."""
    if not os.path.isfile(PKG_BOOTSTRAP_V1_TOOL):
        pytest.skip(f"Bootstrap tool missing: {PKG_BOOTSTRAP_V1_TOOL}")
    os.makedirs(os.path.dirname(FS_EXTERNAL_DISK_IMG), exist_ok=True)

    try:
        result = subprocess.run(
            [
                sys.executable,
                PKG_BOOTSTRAP_V1_TOOL,
                "--pkg-out",
                PKG_V1_PATH,
                "--repo-out",
                PKG_REPO_V1_PATH,
                "--disk-out",
                FS_EXTERNAL_DISK_IMG,
                "--app-name",
                "hello",
                "--app-version",
                "1.0.0",
                "--app-message",
                "APP: hello world\n",
            ],
            capture_output=True,
            text=True,
            timeout=QEMU_TIMEOUT,
        )
    except subprocess.TimeoutExpired:
        pytest.fail("pkg_bootstrap_v1.py timed out")

    if result.returncode != 0:
        pytest.fail(
            "pkg_bootstrap_v1.py failed:\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )

    if not os.path.isfile(FS_EXTERNAL_DISK_IMG):
        pytest.fail(f"External package disk image not created: {FS_EXTERNAL_DISK_IMG}")

    return FS_EXTERNAL_DISK_IMG


@pytest.fixture
def qemu_serial_pkg_external(external_pkg_disk_img):
    """Boot the external-package disk image against the pkg hash kernel ISO."""
    if not os.path.isfile(ISO_PKG_HASH_PATH):
        pytest.skip(f"ISO not built: {ISO_PKG_HASH_PATH}")
    return _boot_iso_with_disk(ISO_PKG_HASH_PATH, external_pkg_disk_img)


def _find_free_udp_port():
    """Find a free UDP port on localhost."""
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def _boot_iso_with_net(iso_path):
    """Boot an ISO in QEMU with virtio-net and inject a UDP echo packet."""
    assert os.path.isfile(iso_path), f"ISO not found: {iso_path}"
    if not QEMU_BIN:
        pytest.skip("qemu-system-x86_64 not found (set QEMU_BIN or install QEMU)")

    udp_port = _find_free_udp_port()

    proc = subprocess.Popen(
        [
            QEMU_BIN,
            "-machine", "q35",
            "-cpu", "qemu64",
            "-m", "128",
            "-serial", "stdio",
            "-display", "none",
            "-no-reboot",
            "-device", "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-cdrom", iso_path,
            "-netdev", f"user,id=n0,hostfwd=udp::{udp_port}-10.0.2.15:7",
            "-device", "virtio-net-pci,netdev=n0,disable-modern=on",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    def _send_udp():
        """Send UDP packets after a boot delay to trigger the echo."""
        time.sleep(3)
        for _ in range(5):
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
                sock.sendto(b"echo test", ("127.0.0.1", udp_port))
                sock.close()
            except OSError:
                pass
            time.sleep(0.5)

    sender = threading.Thread(target=_send_udp, daemon=True)
    sender.start()

    try:
        stdout, stderr = proc.communicate(timeout=NET_TIMEOUT)
    except subprocess.TimeoutExpired:
        proc.kill()
        stdout, stderr = proc.communicate()
        stdout_str = stdout.decode("utf-8", errors="replace") if stdout else ""
        pytest.fail(
            f"QEMU timed out ({NET_TIMEOUT}s). Captured serial:\n{stdout_str}"
        )

    return subprocess.CompletedProcess(
        args=proc.args,
        returncode=proc.returncode,
        stdout=stdout.decode("utf-8", errors="replace") if stdout else "",
        stderr=stderr.decode("utf-8", errors="replace") if stderr else "",
    )


@pytest.fixture
def qemu_serial_net():
    """Boot the VirtIO net test OS image with networking."""
    if not os.path.isfile(ISO_NET_PATH):
        pytest.skip(f"ISO not built: {ISO_NET_PATH}")
    return _boot_iso_with_net(ISO_NET_PATH)


@pytest.fixture
def qemu_serial_go():
    """Boot the G1 TinyGo user-space test OS image."""
    if not os.path.isfile(ISO_GO_PATH):
        pytest.skip(f"ISO not built: {ISO_GO_PATH}")
    return _boot_iso(ISO_GO_PATH)


@pytest.fixture
def qemu_serial_go_std():
    """Boot the G2 standard-Go scaffold OS image."""
    if not os.path.isfile(ISO_GO_STD_PATH):
        pytest.skip(
            "G2 prep: standard-Go image not built yet "
            f"(expected {ISO_GO_STD_PATH})"
        )
    return _boot_iso(ISO_GO_STD_PATH)
