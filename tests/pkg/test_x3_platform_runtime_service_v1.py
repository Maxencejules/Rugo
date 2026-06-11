"""X3 runtime-backed package/storage service checks on the booted Go lane."""

from __future__ import annotations

import hashlib
from pathlib import Path


PKG_STATE_RECORD_MAGIC = 0x504B_4731
PLATFORM_RECORD_MAGIC = 0x5046_5431
PKG_STATE_SECTOR = 10
PLATFORM_SECTOR = 11

META_PAYLOAD_V1 = "stable|seq=1|key=1|catalog=3|pkgs=base-shell@1.0.0,net-tools@1.0.0"
META_PAYLOAD_V2 = "stable|seq=2|key=2|catalog=4|pkgs=base-shell@1.1.0,net-tools@1.0.0,media-suite@2.0.0"
XATTR_PAYLOAD_V1 = "channel=stable"
XATTR_PAYLOAD_V2 = "channel=stable-v2"


def _read_sector(disk_path: str, sector: int) -> bytes:
    image = Path(disk_path).read_bytes()
    start = sector * 512
    return image[start : start + 512]


def _parse_record(record: bytes) -> dict[str, int | bytes]:
    length = int.from_bytes(record[8:12], "little")
    return {
        "magic": int.from_bytes(record[0:4], "little"),
        "flags": int.from_bytes(record[4:8], "little"),
        "length": length,
        "seq": int.from_bytes(record[12:16], "little"),
        "payload": record[16 : 16 + length],
    }


def _prefix(payload: str) -> bytes:
    return hashlib.sha256(payload.encode("utf-8")).digest()[:8]


def test_x3_runtime_service_exercises_signed_updates_and_platform_features(
    qemu_go_c4_runtime,
    find_in_order,
):
    boot, disk_path = qemu_go_c4_runtime

    first = boot().stdout
    find_in_order(
        first,
        [
            "PKGSVC: start",
            "PKGSVC: ready",
            "UPD3: metadata ok",
            "CAT3: catalog ok",
            "CAT3: install base ok",
            "CAT3: install net ok",
            "CAT3: telemetry ok",
            "STORX3: snapshot ok",
            "STORX3: xattr ok",
            "STORX3: cap ok",
            "GOSH: pkg ok",
            "PKGSVC: stop",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )
    assert "UPD3: rotate ok" not in first
    assert "STORX3: resize ok" not in first
    assert "STORX3: reflink ok" not in first

    pkg_record = _parse_record(_read_sector(disk_path, PKG_STATE_SECTOR))
    platform_record = _parse_record(_read_sector(disk_path, PLATFORM_SECTOR))
    assert pkg_record["magic"] == PKG_STATE_RECORD_MAGIC
    assert pkg_record["flags"] == 0
    assert pkg_record["length"] == 28
    assert pkg_record["seq"] == 1
    pkg_payload = pkg_record["payload"]
    assert pkg_payload[0:4] == b"PST1"
    assert pkg_payload[4] == 1
    assert pkg_payload[5] == 1
    assert pkg_payload[6] == 1
    assert pkg_payload[7] == 1
    assert pkg_payload[8] == 1
    assert pkg_payload[9] == 3
    assert pkg_payload[10] == 0x03
    assert pkg_payload[11] == 2
    assert pkg_payload[12:20] == _prefix(META_PAYLOAD_V1)
    assert pkg_payload[20:28] == _prefix(META_PAYLOAD_V1)

    assert platform_record["magic"] == PLATFORM_RECORD_MAGIC
    assert platform_record["flags"] == 0
    assert platform_record["length"] == 28
    assert platform_record["seq"] == 1
    platform_payload = platform_record["payload"]
    assert platform_payload[0:4] == b"PLT1"
    assert platform_payload[4] == 1
    assert platform_payload[5] == 64
    assert platform_payload[6] == len(XATTR_PAYLOAD_V1)
    assert platform_payload[7] == 0
    assert platform_payload[8] == 0x15
    assert platform_payload[9] == 0x03
    assert platform_payload[10:18] == _prefix(META_PAYLOAD_V1)
    assert platform_payload[18:26] == _prefix(XATTR_PAYLOAD_V1)

    second = boot().stdout
    find_in_order(
        second,
        [
            "PKGSVC: start",
            "PKGSVC: ready",
            "UPD3: rotate ok",
            "UPD3: metadata ok",
            "CAT3: catalog ok",
            "CAT3: stage ok",
            "CAT3: install media ok",
            "CAT3: telemetry ok",
            "STORX3: resize ok",
            "STORX3: reflink ok",
            "STORX3: xattr ok",
            "STORX3: cap ok",
            "UPD3: rollback ok",
            "UPD3: apply ok",
            "GOSH: pkg ok",
            "PKGSVC: stop",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )

    pkg_record = _parse_record(_read_sector(disk_path, PKG_STATE_SECTOR))
    platform_record = _parse_record(_read_sector(disk_path, PLATFORM_SECTOR))
    assert pkg_record["seq"] == 2
    pkg_payload = pkg_record["payload"]
    assert pkg_payload[4] == 2
    assert pkg_payload[5] == 2
    assert pkg_payload[6] == 2
    assert pkg_payload[7] == 2
    assert pkg_payload[8] == 1
    assert pkg_payload[9] == 4
    assert pkg_payload[10] == 0x07
    assert pkg_payload[11] == 5
    assert pkg_payload[12:20] == _prefix(META_PAYLOAD_V2)
    assert pkg_payload[20:28] == _prefix(META_PAYLOAD_V2)

    assert platform_record["seq"] == 2
    platform_payload = platform_record["payload"]
    assert platform_payload[4] == 2
    assert platform_payload[5] == 96
    assert platform_payload[6] == len(XATTR_PAYLOAD_V2)
    assert platform_payload[7] == 1
    assert platform_payload[8] == 0x1F
    assert platform_payload[9] == 0x07
    assert platform_payload[10:18] == _prefix(META_PAYLOAD_V2)
    assert platform_payload[18:26] == _prefix(XATTR_PAYLOAD_V2)
