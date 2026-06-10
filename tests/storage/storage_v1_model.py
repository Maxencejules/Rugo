"""Deterministic reference model for M13 storage reliability semantics."""

from __future__ import annotations

from dataclasses import dataclass


def checksum32(data: bytes) -> int:
    return sum(data) & 0xFFFF_FFFF


@dataclass
class WriteOrderingModel:
    data_written: bool = False
    data_barrier_done: bool = False
    metadata_written: bool = False
    metadata_barrier_done: bool = False
    clean_marker_written: bool = False
    ordering_valid: bool = True

    def write_data(self) -> None:
        self.data_written = True

    def data_barrier(self) -> None:
        if not self.data_written:
            self.ordering_valid = False
        self.data_barrier_done = True

    def write_metadata(self) -> None:
        if not self.data_barrier_done:
            self.ordering_valid = False
        self.metadata_written = True

    def metadata_barrier(self) -> None:
        if not self.metadata_written:
            self.ordering_valid = False
        self.metadata_barrier_done = True

    def write_clean_marker(self) -> None:
        if not self.metadata_barrier_done:
            self.ordering_valid = False
        self.clean_marker_written = True


@dataclass
class DurabilityModel:
    data_pending: bool = False
    metadata_pending: bool = False
    data_durable: bool = False
    metadata_durable: bool = False

    def write_data(self) -> None:
        self.data_pending = True

    def write_metadata(self) -> None:
        self.metadata_pending = True

    def fdatasync(self) -> None:
        if self.data_pending:
            self.data_durable = True

    def fsync(self) -> None:
        if self.data_pending:
            self.data_durable = True
        if self.metadata_pending:
            self.metadata_durable = True

    def crash(self) -> tuple[bool, bool]:
        # Return surviving data/metadata durability outcomes.
        return self.data_durable, self.metadata_durable


def parse_simplefs_superblock(image: bytes) -> dict[str, int | bool]:
    if len(image) < 16:
        return {
            "magic_ok": False,
            "file_count_ok": False,
            "data_start_ok": False,
            "next_free_ok": False,
            "mountable": False,
        }

    magic = int.from_bytes(image[0:4], "little")
    file_count = int.from_bytes(image[4:8], "little")
    data_start = int.from_bytes(image[8:12], "little")
    next_free = int.from_bytes(image[12:16], "little")

    checks = {
        "magic_ok": magic == 0x53465331,
        "file_count_ok": file_count <= 16,
        "data_start_ok": data_start >= 2,
        "next_free_ok": next_free >= data_start,
    }
    checks["mountable"] = all(bool(v) for v in checks.values())
    return checks
