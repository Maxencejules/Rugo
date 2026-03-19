"""C4 runtime acceptance on the native NVMe storage lane."""

from pathlib import Path


R4_STORAGE_JOURNAL_MAGIC = 0x4A52_4E31
R4_STORAGE_STATE_MAGIC = 0x5354_4131
R4_STORAGE_JOURNAL_SECTOR = 8
R4_STORAGE_STATE_SECTOR = 9


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def _read_sector(disk_path: str, sector: int) -> bytes:
    image = Path(disk_path).read_bytes()
    start = sector * 512
    return image[start : start + 512]


def _parse_record(record: bytes) -> dict[str, int | bytes]:
    return {
        "magic": int.from_bytes(record[0:4], "little"),
        "flags": int.from_bytes(record[4:8], "little"),
        "length": int.from_bytes(record[8:12], "little"),
        "seq": int.from_bytes(record[12:16], "little"),
        "payload": record[16 : 16 + int.from_bytes(record[8:12], "little")],
    }


def test_connected_runtime_c4_native_replays_storage_and_flushes_on_nvme(
    qemu_go_c4_runtime_native,
):
    boot, disk_path = qemu_go_c4_runtime_native

    first = boot().stdout
    _find_in_order(
        first,
        [
            "IRQ: vector bound vec=64",
            "BAR: map ok bar=0 bytes=65536",
            "DRV: bind driver=nvme",
            "FW: allow signed driver=nvme",
            "NVME: ready",
            "NVME: identify ok",
            "NVME: io queue ok",
            "STORC4: block ready driver=nvme",
            "NETC4: nic ready",
            "GOSH: diag ok",
            "STORC4: journal staged",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )
    assert "RECOV: replay ok" not in first
    assert "BLK: fua ok" not in first
    assert "BLK: flush ordered" not in first

    first_journal = _parse_record(_read_sector(disk_path, R4_STORAGE_JOURNAL_SECTOR))
    first_state = _parse_record(_read_sector(disk_path, R4_STORAGE_STATE_SECTOR))
    assert first_journal["magic"] == R4_STORAGE_JOURNAL_MAGIC
    assert first_journal["flags"] == 1
    assert first_journal["payload"] == b"c4-recover"
    assert first_state["magic"] == 0

    second = boot().stdout
    _find_in_order(
        second,
        [
            "IRQ: vector bound vec=64",
            "BAR: map ok bar=0 bytes=65536",
            "DRV: bind driver=nvme",
            "FW: allow signed driver=nvme",
            "NVME: ready",
            "NVME: identify ok",
            "NVME: io queue ok",
            "STORC4: block ready driver=nvme",
            "RECOV: replay ok",
            "STORC4: state ok",
            "BLK: fua ok",
            "BLK: flush ordered",
            "STORC4: fsync ok",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )
    assert "STORC4: journal staged" not in second

    second_journal = _parse_record(_read_sector(disk_path, R4_STORAGE_JOURNAL_SECTOR))
    second_state = _parse_record(_read_sector(disk_path, R4_STORAGE_STATE_SECTOR))
    assert second_journal["magic"] == R4_STORAGE_JOURNAL_MAGIC
    assert second_journal["flags"] == 0
    assert second_journal["length"] == 0
    assert second_state["magic"] == R4_STORAGE_STATE_MAGIC
    assert second_state["payload"] == b"c4-fsync"
