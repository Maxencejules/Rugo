"""C4 runtime acceptance: durable storage and connected sockets run in image-go."""

from pathlib import Path


R4_STORAGE_JOURNAL_MAGIC = 0x4A52_4E31
R4_STORAGE_STATE_MAGIC = 0x5354_4131
R4_STORAGE_JOURNAL_SECTOR = 8
R4_STORAGE_STATE_SECTOR = 9


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


def test_connected_runtime_c4_replays_storage_and_exercises_ipv6_sockets(
    qemu_go_c4_runtime,
    find_in_order,
):
    boot, disk_path = qemu_go_c4_runtime

    first = boot().stdout
    find_in_order(
        first,
        [
            "STORC4: block ready",
            "NETC4: nic ready",
            "GOSH: diag ok",
            "STORC4: journal staged",
            "NETC4: ifcfg ok",
            "NETC4: route ok",
            "NETC4: listen ok",
            "NETC4: connect ok",
            "NETC4: accept ok",
            "NETC4: recv ok",
            "NETC4: reply ok",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )
    assert "RECOV: replay ok" not in first
    assert "STORC4: state ok" not in first
    assert "STORC4: fsync ok" not in first
    assert "BLK: flush ordered" not in first

    first_journal = _parse_record(_read_sector(disk_path, R4_STORAGE_JOURNAL_SECTOR))
    first_state = _parse_record(_read_sector(disk_path, R4_STORAGE_STATE_SECTOR))
    assert first_journal["magic"] == R4_STORAGE_JOURNAL_MAGIC
    assert first_journal["flags"] == 1
    assert first_journal["payload"] == b"c4-recover"
    assert first_state["magic"] == 0

    second = boot().stdout
    find_in_order(
        second,
        [
            "STORC4: block ready",
            "RECOV: replay ok",
            "NETC4: nic ready",
            "GOSH: diag ok",
            "STORC4: state ok",
            "BLK: flush ordered",
            "STORC4: fsync ok",
            "NETC4: ifcfg ok",
            "NETC4: route ok",
            "NETC4: listen ok",
            "NETC4: connect ok",
            "NETC4: accept ok",
            "NETC4: recv ok",
            "NETC4: reply ok",
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
