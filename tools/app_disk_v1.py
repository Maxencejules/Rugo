#!/usr/bin/env python3
"""Write the exec app region onto a boot disk image.

Layout (all offsets relative to --base-sector, default 64, clear of the
runtime-state sectors 8..11):
  base+0: SimpleFS superblock  magic "SF31" u32 | file_count u32 |
          data_start u32 (absolute sector) | next_free u32
  base+1..base+2: file table, 32 entries x 32 bytes (two sectors):
          name[24] zero-padded | start_sector u32 (absolute) | size u32
  base+3...: file payloads, each PKG v1-framed:
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
    parser.add_argument(
        "--app",
        action="append",
        default=[],
        metavar="NAME=ELF",
        help="app to install (repeatable)",
    )
    parser.add_argument("--elf", help="single-app compatibility: ELF payload")
    parser.add_argument("--name", default="base-shell")
    parser.add_argument("--base-sector", type=int, default=64)
    args = parser.parse_args()

    apps = []
    for spec in args.app:
        name, _, elf_path = spec.partition("=")
        apps.append((name, Path(elf_path).read_bytes()))
    if args.elf:
        apps.append((args.name, Path(args.elf).read_bytes()))
    if not apps or len(apps) > 48:
        raise SystemExit("app-disk: between 1 and 48 apps required")

    # Superblock at `base`, then a 3-sector (48-entry x 32-byte) file table,
    # then the packed app payloads. Kept in sync with sys_spawn_v1's reader.
    base = args.base_sector
    data_sector = base + 4
    table = bytearray(3 * SECTOR)
    payloads = bytearray()
    cursor = data_sector
    for index, (name, elf) in enumerate(apps):
        pkg = build_pkg(name, elf)
        pkg_sectors = (len(pkg) + SECTOR - 1) // SECTOR
        entry = name.encode("ascii")[:24].ljust(24, b"\x00")
        entry += struct.pack("<II", cursor, len(pkg))
        table[index * 32 : (index + 1) * 32] = entry
        payloads += pkg.ljust(pkg_sectors * SECTOR, b"\x00")
        cursor += pkg_sectors

    superblock = struct.pack("<IIII", SIMPLEFS_MAGIC, len(apps), data_sector, cursor)

    disk_path = Path(args.disk)
    needed = cursor * SECTOR
    disk = bytearray(disk_path.read_bytes()) if disk_path.is_file() else bytearray()
    if len(disk) < needed:
        disk.extend(b"\x00" * (needed - len(disk)))

    disk[base * SECTOR : base * SECTOR + SECTOR] = superblock.ljust(SECTOR, b"\x00")
    disk[(base + 1) * SECTOR : (base + 4) * SECTOR] = table
    disk[data_sector * SECTOR : data_sector * SECTOR + len(payloads)] = payloads

    disk_path.write_bytes(bytes(disk))
    names = ",".join(name for name, _ in apps)
    print(f"app-disk: {names} at sector {data_sector}+ of {disk_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
