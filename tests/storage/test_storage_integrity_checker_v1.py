"""M13 acceptance: integrity checker primitives."""

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "out"

sys.path.append(str(ROOT / "tools"))
import storage_recover_v1 as recover  # noqa: E402

sys.path.append(str(Path(__file__).resolve().parent))
from storage_v1_model import checksum32  # noqa: E402


def _ensure_fs_image_bytes() -> bytes:
    image = OUT / "fs-test.img"
    if not image.is_file():
        mkfs = ROOT / "tools" / "mkfs.py"
        OUT.mkdir(parents=True, exist_ok=True)
        proc = subprocess.run(
            [sys.executable, str(mkfs), str(image)],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            raise AssertionError(
                "mkfs.py failed while preparing fs-test image:\n"
                f"stdout:\n{proc.stdout}\n"
                f"stderr:\n{proc.stderr}"
            )
    return image.read_bytes()


def test_checksum_detects_corruption():
    base = _ensure_fs_image_bytes()
    changed = bytearray(base)
    changed[0] ^= 0xFF
    assert checksum32(base) != checksum32(bytes(changed))


def test_superblock_corruption_breaks_mountability():
    base = _ensure_fs_image_bytes()
    broken = bytearray(base)
    broken[0:4] = b"\x00\x00\x00\x00"

    report = recover.analyze_image_bytes(bytes(broken))
    assert report["checks"]["magic_ok"] is False
    assert report["mountable"] is False
