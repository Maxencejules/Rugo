"""M18 acceptance: metadata integrity and v2 doc contract checks."""

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "out"

sys.path.append(str(ROOT / "tools"))
import storage_recover_v2 as recover  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


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


def test_metadata_fingerprint_detects_corruption():
    base = _ensure_fs_image_bytes()
    changed = bytearray(base)
    changed[8] ^= 0x7F
    assert recover.metadata_fingerprint(base) != recover.metadata_fingerprint(
        bytes(changed)
    )


def test_next_free_out_of_bounds_breaks_mountability():
    base = _ensure_fs_image_bytes()
    broken = bytearray(base)
    broken[12:16] = (0xFFFF_FFFF).to_bytes(4, "little")

    report = recover.analyze_image_bytes(bytes(broken))
    assert report["checks"]["next_free_in_bounds"] is False
    assert report["mountable"] is False


def test_storage_v2_doc_contract_tokens():
    fs_v2 = _read("docs/storage/fs_v2.md")
    durability = _read("docs/storage/durability_model_v2.md")
    ordering = _read("docs/storage/write_ordering_policy_v2.md")

    assert "Status: active release gate" in fs_v2
    assert "Durability classes" in fs_v2
    assert "`fdatasync`" in durability
    assert "journal commit marker" in ordering
