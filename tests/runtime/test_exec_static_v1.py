"""Real sys_spawn containment for checksum-valid malformed ET_EXEC packages.

The negative ELF bytes are synthetic inputs, never executable demo results.
Positive controls run the existing assembled page3probe and linked PIE shell.
"""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import struct
import subprocess
import sys

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import conftest  # noqa: E402

BASE = 0x01400000
ARGS = 0x017FF000


def _image(count: int = 1) -> bytearray:
    image = bytearray(320)
    image[:7] = b"\x7fELF\x02\x01\x01"
    struct.pack_into("<HHIQQ", image, 16, 2, 62, 1, BASE, 64)
    struct.pack_into("<HHH", image, 52, 64, 56, count)
    for index in range(count):
        struct.pack_into(
            "<IIQQQQQQ",
            image,
            64 + 56 * index,
            1,
            5,
            256 + 8 * index,
            BASE + 4096 * index,
            0,
            4,
            8,
            1,
        )
        image[256 + 8 * index : 260 + 8 * index] = b"\x90\x90\x90\xc3"
    return image


def _malformed_images() -> dict[str, bytes]:
    images = {}
    image = _image()
    struct.pack_into("<Q", image, 32, 2**64 - 1)
    images["elf-phoff"] = bytes(image)
    image = _image()
    struct.pack_into("<Q", image, 72, 2**64 - 1)
    images["elf-file"] = bytes(image)
    image = _image()
    struct.pack_into("<I", image, 64, 0)
    images["elf-empty"] = bytes(image)
    image = _image()
    struct.pack_into("<Q", image, 24, BASE + 4096)
    images["elf-entry"] = bytes(image)
    image = _image()
    struct.pack_into("<H", image, 18, 183)
    images["elf-machine"] = bytes(image)
    image = _image(2)
    struct.pack_into("<Q", image, 136, ARGS)
    images["elf-late"] = bytes(image)
    image = _image()
    struct.pack_into("<Q", image, 80, ARGS)
    struct.pack_into("<Q", image, 24, ARGS)
    images["elf-args"] = bytes(image)
    return images


def _write_region(directory: Path, payloads: dict[str, bytes]) -> Path:
    disk = directory / "exec-static.img"
    disk.write_bytes(bytes(1024 * 1024))
    command = [sys.executable, conftest.APP_DISK_V1_TOOL, "--disk", str(disk)]
    for name, payload in payloads.items():
        elf = directory / f"{name}.elf"
        elf.write_bytes(payload)
        command += ["--app", f"{name}={elf}"]
    subprocess.run(command, check=True, capture_output=True, text=True, timeout=30)
    return disk


def test_negative_fixtures_are_real_checksum_valid_disk_packages(tmp_path):
    """Prevent badhash from accidentally satisfying the ELF rejection test."""
    payloads = _malformed_images()
    disk = _write_region(tmp_path, payloads).read_bytes()
    assert struct.unpack_from("<I", disk, 64 * 512 + 4)[0] == len(payloads)
    for index, (name, payload) in enumerate(payloads.items()):
        table = 65 * 512 + index * 32
        sector, size = struct.unpack_from("<II", disk, table + 24)
        package = disk[sector * 512 : sector * 512 + size]
        assert disk[table : table + 24].rstrip(b"\x00").decode("ascii") == name
        assert struct.unpack_from("<II", package)[0] == 0x01474B50
        assert struct.unpack_from("<II", package)[1] == len(payload)
        assert package[32:64] == hashlib.sha256(payload).digest()
        assert package[64:] == payload


def _required(path: Path) -> bytes:
    if not path.is_file():
        message = f"native static ELF gate needs built artifact: {path}"
        if os.environ.get("RUGO_EXEC_STATIC_STRICT") == "1":
            pytest.fail(message)
        pytest.skip(message)
    return path.read_bytes()


def test_sys_spawn_rejects_malformed_static_apps_then_runs_static_and_pie(
    tmp_path, find_in_order
):
    root = Path(conftest.REPO_ROOT)
    _required(Path(conftest.ISO_GO_PATH))
    if not conftest.QEMU_BIN:
        if os.environ.get("RUGO_EXEC_STATIC_STRICT") == "1":
            pytest.fail("native static ELF gate requires QEMU_BIN")
        pytest.skip("QEMU_BIN not available")
    static = _required(root / "out" / "app-page3probe.elf")
    pie = _required(root / "out" / "app-base-shell.elf")
    # These controls protect both product loader lanes from fixture drift.
    assert struct.unpack_from("<H", static, 16)[0] == 2
    assert struct.unpack_from("<H", pie, 16)[0] == 3
    negatives = _malformed_images()
    disk = _write_region(
        tmp_path, {**negatives, "page3probe": static, "base-shell": pie}
    )
    commands = "".join(f"probe {name}\n" for name in negatives)
    # `run` checks installed-package state populated by the `pkg` command;
    # generic malformed probes intentionally exercise sys_spawn directly.
    commands += "probe page3probe\npkg\nrun base-shell\nhealth\nshutdown\n"
    result = conftest._boot_iso_with_disk_and_net(
        conftest.ISO_GO_PATH, str(disk), input_text=commands
    )
    proof = root / "out" / "exec-static-v1"
    proof.mkdir(parents=True, exist_ok=True)
    (proof / "qemu-serial.txt").write_text(result.stdout, encoding="utf-8")
    out = result.stdout
    find_in_order(
        out,
        [
            *(f"EXEC: {name} badelf" for name in negatives),
            "EXEC: page3probe ok",
            "PAGE3: ok",
            "GOSH: pkg ok",
            "EXEC: base-shell ok",
            "BASESH: hello from disk",
            "APP: base-shell ok",
            "GOINIT: result shutdown-clean",
            "RUGO: halt ok",
        ],
    )
    for name in negatives:
        assert out.count(f"EXEC: {name} badelf") == 1
        assert f"EXEC: {name} ok" not in out
    for unexpected in [
        "badhash",
        "badpkg",
        "PAGE3: FAIL",
        "USERPF",
        "PANIC:",
        "GOINIT: err",
    ]:
        assert unexpected not in out
