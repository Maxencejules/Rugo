#!/usr/bin/env python3
"""Write a signed package repository onto a boot-disk scratch region.

Full-OS guide Part V.11 (package manager). The kernel's pkg_manager_selftest
reads this repo, verifies the HMAC-signed index, selects a package by name,
verifies that package's SHA-256, and installs it.

On-disk layout (LBAs in the free 12..63 scratch gap, clear of pkgfetch@16 and
pkg-install@21):
  LBA 24: index sector (512 bytes)
      [0..4]   magic "RPKG"
      [4..8]   package count u32 LE
      [8..40]  index signature = HMAC-SHA256(KEY, SHA-256(entries))
      [40..]   entries, 64 bytes each:
          [0..24]  name (ascii, NUL-padded)
          [24..28] payload start LBA u32 LE
          [28..32] payload length u32 LE
          [32..64] SHA-256(payload)
  LBA 25..: package payloads, each padded to a whole sector.

Must stay in lockstep with pkg_manager_selftest in kernel_rs/src/net.rs.
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import struct
from pathlib import Path

SECTOR = 512
INDEX_LBA = 24
PAYLOAD_LBA = 25
KEY = b"rugo-repo-index-key-v1"
ENTRY_SIZE = 64
MAX_ENTRIES = 7  # 40 + 7*64 = 488 <= 512


def build_payload(name: str, length: int) -> bytes:
    # Deterministic per-package bytes so the kernel's SHA-256 check is exact.
    seed = (sum(name.encode("ascii")) & 0xFF) or 1
    return bytes(((i * seed + 0x11) & 0xFF) for i in range(length))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--disk", required=True)
    ap.add_argument(
        "--pkg",
        action="append",
        default=[],
        metavar="NAME:LEN",
        help="package name and payload length in bytes (repeatable)",
    )
    args = ap.parse_args()

    pkgs = []
    for spec in args.pkg:
        name, _, length = spec.partition(":")
        pkgs.append((name, int(length)))
    if not pkgs:
        pkgs = [("calc", 700), ("edit", 300), ("term", 512)]
    if len(pkgs) > MAX_ENTRIES:
        raise SystemExit(f"pkg-repo: at most {MAX_ENTRIES} packages")

    entries = bytearray()
    payload_blob = bytearray()
    lba = PAYLOAD_LBA
    for name, length in pkgs:
        if len(name.encode("ascii")) > 24:
            raise SystemExit(f"pkg-repo: name too long: {name}")
        if length < 1 or length > 1024:
            raise SystemExit(f"pkg-repo: payload length out of range: {length}")
        payload = build_payload(name, length)
        sectors = (length + SECTOR - 1) // SECTOR
        entry = name.encode("ascii").ljust(24, b"\x00")
        entry += struct.pack("<II", lba, length)
        entry += hashlib.sha256(payload).digest()
        assert len(entry) == ENTRY_SIZE
        entries += entry
        payload_blob += payload.ljust(sectors * SECTOR, b"\x00")
        lba += sectors

    index_digest = hashlib.sha256(bytes(entries)).digest()
    index_sig = hmac.new(KEY, index_digest, hashlib.sha256).digest()
    index = bytearray(SECTOR)
    index[0:4] = b"RPKG"
    struct.pack_into("<I", index, 4, len(pkgs))
    index[8:40] = index_sig
    index[40:40 + len(entries)] = entries

    disk_path = Path(args.disk)
    disk = bytearray(disk_path.read_bytes()) if disk_path.is_file() else bytearray()
    needed = (PAYLOAD_LBA + (len(payload_blob) // SECTOR) + 1) * SECTOR
    if len(disk) < needed:
        disk.extend(b"\x00" * (needed - len(disk)))
    disk[INDEX_LBA * SECTOR:INDEX_LBA * SECTOR + SECTOR] = index
    disk[PAYLOAD_LBA * SECTOR:PAYLOAD_LBA * SECTOR + len(payload_blob)] = payload_blob
    disk_path.write_bytes(bytes(disk))
    names = ",".join(n for n, _ in pkgs)
    print(f"pkg-repo: {len(pkgs)} packages ({names}) -> index LBA {INDEX_LBA} of {disk_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
