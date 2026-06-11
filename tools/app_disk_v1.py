#!/usr/bin/env python3
"""Write the exec app region onto a boot disk image.

Layout (all offsets relative to --base-sector, default 64, clear of the
runtime-state sectors 8..11):
  base+0: SimpleFS superblock  magic "SF31" u32 | file_count u32 |
          data_start u32 (absolute sector) | next_free u32
  base+1: file table, 16 entries x 32 bytes:
          name[24] zero-padded | start_sector u32 (absolute) | size u32
  base+2...: file payloads, each PKG v1-framed:
          magic u32 (0x01474B50) | bin_size u32 | name[24] | sha256[32]
          followed by bin_size payload bytes.

The kernel's sys_spawn reads this region, verifies the SHA-256, and loads
the contained ELF into the exec app window.
"""

from __future__ import annotations

import argparse
import hashlib
import struct
from pathlib import Path

SECTOR = 512
SIMPLEFS_MAGIC = 0x53465331  # "SFS1" little-endian bytes "1SFS" on disk
PKG_MAGIC_V1 = 0x01474B50


def build_pkg(name: str, payload: bytes) -> bytes:
    digest = hashlib.sha256(payload).digest()
    header = struct.pack("<II", PKG_MAGIC_V1, len(payload))
    header += name.encode("ascii")[:24].ljust(24, b"\x00")
    header += digest
    assert len(header) == 64
    return header + payload


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--disk", required=True, help="disk image to patch")
    parser.add_argument("--elf", required=True, help="app ELF payload")
    parser.add_argument("--name", default="base-shell")
    parser.add_argument("--base-sector", type=int, default=64)
    args = parser.parse_args()

    disk_path = Path(args.disk)
    elf = Path(args.elf).read_bytes()
    pkg = build_pkg(args.name, elf)

    base = args.base_sector
    data_sector = base + 2
    pkg_sectors = (len(pkg) + SECTOR - 1) // SECTOR

    superblock = struct.pack(
        "<IIII", SIMPLEFS_MAGIC, 1, data_sector, data_sector + pkg_sectors
    )

    entry = args.name.encode("ascii")[:24].ljust(24, b"\x00")
    entry += struct.pack("<II", data_sector, len(pkg))
    table = entry.ljust(SECTOR, b"\x00")

    needed = (data_sector + pkg_sectors) * SECTOR
    disk = bytearray(disk_path.read_bytes()) if disk_path.is_file() else bytearray()
    if len(disk) < needed:
        disk.extend(b"\x00" * (needed - len(disk)))

    disk[base * SECTOR : base * SECTOR + SECTOR] = superblock.ljust(SECTOR, b"\x00")
    disk[(base + 1) * SECTOR : (base + 2) * SECTOR] = table
    payload = pkg.ljust(pkg_sectors * SECTOR, b"\x00")
    disk[data_sector * SECTOR : data_sector * SECTOR + len(payload)] = payload

    disk_path.write_bytes(bytes(disk))
    print(
        f"app-disk: {args.name} ({len(elf)} bytes ELF) at sector {data_sector} "
        f"of {disk_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
