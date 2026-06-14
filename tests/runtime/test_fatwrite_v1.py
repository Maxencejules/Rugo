# Full-OS guide Part II.5 acceptance: FAT16 file WRITE.
#
# sys_sysinfo(op=11) allocates a free cluster, marks it EOC in every FAT copy,
# writes a single-cluster file ("WRTEST.TXT") into a free root-directory entry of
# the FAT volume, then reads it back via the existing reader and confirms a
# byte-exact round-trip ("FATWR: write+read ok"). fatwrprobe triggers it. The
# crafted volume already holds HELLO.TXT (cluster 2 / root slot 0), so the write
# must allocate cluster 3 and root slot 1 without disturbing it.

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import conftest  # noqa: E402
from test_fat16_v1 import _fat16_volume, SEC  # noqa: E402


def test_fat16_write_and_readback(qemu_go_c4_runtime, find_in_order):
    boot, disk_path = qemu_go_c4_runtime

    disk = bytearray(4 * 1024 * 1024)  # 4 MiB
    vol = _fat16_volume()
    disk[2048 * SEC:2048 * SEC + len(vol)] = vol
    with open(disk_path, "wb") as f:
        f.write(disk)

    out = boot("probe fatwrprobe\nshutdown\n").stdout

    find_in_order(out, [
        "FATWR: write+read ok",
        "FATWRPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "FATWRPROBE: FAIL" not in out

    # Host-side: the new file really landed in the FAT root directory, alongside
    # the pre-existing HELLO.TXT.
    with open(disk_path, "rb") as f:
        image = f.read()
    assert b"WRTEST  TXT" in image, "WRTEST.TXT directory entry not written"
    assert b"HELLO   TXT" in image, "pre-existing HELLO.TXT was clobbered"
