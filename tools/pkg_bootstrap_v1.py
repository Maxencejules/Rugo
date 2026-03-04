#!/usr/bin/env python3
"""Package/repository v1 bootstrap helpers for M8 PR-3.

This tool defines a deterministic package v1 format for external-app bootstrap
workflows and bridges that package into the existing M6 runtime path
(`hello.pkg` inside SimpleFS) so the current kernel can execute the payload in
QEMU.
"""

import argparse
import hashlib
import hmac
import json
import os
import struct
from datetime import datetime, timezone


PKG_V1_MAGIC = b"RPKGV1\x00\x00"
PKG_V0_MAGIC = 0x01474B50  # "PKG\x01"
SFS_MAGIC = 0x53465331  # "SFS1"
DISK_SIZE = 1024 * 1024
SECTOR_SIZE = 512
DEFAULT_REPO_KEY = "rugo-dev-bootstrap-key"


def _canonical_json(obj):
    return json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")


def _sha256_hex(data):
    return hashlib.sha256(data).hexdigest()


def build_debug_write_app(message):
    """Build a tiny x86-64 user payload that sys_debug_write's `message`."""
    if isinstance(message, str):
        msg = message.encode("ascii")
    else:
        msg = bytes(message)

    if not msg:
        raise ValueError("message must not be empty")
    if len(msg) > 0xFFFF_FFFF:
        raise ValueError("message is too large")

    # lea rdi, [rip+12]
    # mov rsi, <len>
    # xor eax, eax
    # int 0x80
    # hlt
    code = bytearray(
        b"\x48\x8D\x3D\x0C\x00\x00\x00"
        b"\x48\xC7\xC6"
        b"\x00\x00\x00\x00"
        b"\x31\xC0\xCD\x80\xF4"
    )
    code[10:14] = struct.pack("<I", len(msg))
    code.extend(msg)
    return bytes(code)


def build_pkg_v1(name, version, payload, abi_profile="compat_profile_v1"):
    """Build a package v1 blob: fixed header + JSON manifest + raw payload."""
    payload = bytes(payload)
    manifest = {
        "abi_profile": abi_profile,
        "name": name,
        "payload_sha256": _sha256_hex(payload),
        "payload_size": len(payload),
        "schema": "rugo.pkg.v1",
        "version": version,
    }
    manifest_blob = _canonical_json(manifest)
    header = struct.pack("<8sII", PKG_V1_MAGIC, len(manifest_blob), len(payload))
    return header + manifest_blob + payload


def parse_pkg_v1(blob):
    """Parse and validate a package v1 blob."""
    if len(blob) < 16:
        raise ValueError("package blob too small")
    magic, manifest_len, payload_len = struct.unpack("<8sII", blob[:16])
    if magic != PKG_V1_MAGIC:
        raise ValueError("bad package magic")
    manifest_end = 16 + manifest_len
    payload_end = manifest_end + payload_len
    if payload_end != len(blob):
        raise ValueError("size mismatch in package blob")

    manifest = json.loads(blob[16:manifest_end].decode("utf-8"))
    payload = blob[manifest_end:payload_end]
    if manifest.get("schema") != "rugo.pkg.v1":
        raise ValueError("unexpected package schema")
    if manifest.get("payload_size") != len(payload):
        raise ValueError("payload_size mismatch")
    if manifest.get("payload_sha256") != _sha256_hex(payload):
        raise ValueError("payload_sha256 mismatch")
    return manifest, payload


def build_repo_metadata_for_pkg(pkg_manifest, pkg_filename, pkg_blob, generated_at):
    """Build unsigned repository metadata for one package."""
    return {
        "generated_at": generated_at,
        "packages": [
            {
                "abi_profile": pkg_manifest["abi_profile"],
                "name": pkg_manifest["name"],
                "pkg_file": pkg_filename,
                "pkg_sha256": _sha256_hex(pkg_blob),
                "pkg_size": len(pkg_blob),
                "version": pkg_manifest["version"],
            }
        ],
        "schema": "rugo.repo.v1",
    }


def sign_repo_metadata(metadata, key, key_id="bootstrap-dev"):
    """Return signed metadata document using deterministic HMAC-SHA256."""
    mac = hmac.new(key.encode("utf-8"), _canonical_json(metadata), hashlib.sha256)
    return {
        "metadata": metadata,
        "signature": {
            "alg": "HMAC-SHA256",
            "key_id": key_id,
            "sig_hex": mac.hexdigest(),
        },
    }


def verify_repo_metadata(signed_doc, key):
    """Verify signed repository metadata."""
    signature = signed_doc.get("signature", {})
    if signature.get("alg") != "HMAC-SHA256":
        return False
    metadata = signed_doc.get("metadata")
    if not isinstance(metadata, dict):
        return False
    expected = hmac.new(
        key.encode("utf-8"), _canonical_json(metadata), hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature.get("sig_hex", ""), expected)


def build_pkg_v0(name, payload):
    """Build runtime-compatible PKG v0 blob (64-byte header + payload)."""
    payload = bytes(payload)
    header = struct.pack("<II", PKG_V0_MAGIC, len(payload))
    header += name.encode("ascii")[:24].ljust(24, b"\x00")
    header += hashlib.sha256(payload).digest()
    return header + payload


def build_simplefs_image(output_path, files):
    """Build a SimpleFS image with provided files.

    `files` is an ordered list of `(filename, data)` tuples.
    """
    if not files:
        raise ValueError("files must not be empty")
    if len(files) > 16:
        raise ValueError("SimpleFS supports at most 16 files")

    disk = bytearray(DISK_SIZE)
    file_count = len(files)
    data_start = 2
    next_sector = data_start

    file_table = bytearray(SECTOR_SIZE)
    for index, (filename, data) in enumerate(files):
        data = bytes(data)
        data_sectors = (len(data) + SECTOR_SIZE - 1) // SECTOR_SIZE
        data_offset = next_sector * SECTOR_SIZE
        disk[data_offset : data_offset + len(data)] = data

        entry = filename.encode("ascii")[:24].ljust(24, b"\x00")
        entry += struct.pack("<II", next_sector, len(data))
        file_table[index * 32 : (index + 1) * 32] = entry
        next_sector += data_sectors

    superblock = struct.pack("<IIII", SFS_MAGIC, file_count, data_start, next_sector)
    disk[0 : len(superblock)] = superblock
    disk[SECTOR_SIZE : 2 * SECTOR_SIZE] = file_table

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    with open(output_path, "wb") as f:
        f.write(disk)


def install_pkg_v1_to_simplefs(pkg_blob, disk_output_path, runtime_filename="hello.pkg"):
    """Bridge package v1 payload to runtime `hello.pkg` layout inside SimpleFS."""
    manifest, payload = parse_pkg_v1(pkg_blob)
    runtime_pkg = build_pkg_v0(manifest["name"], payload)
    build_simplefs_image(disk_output_path, [(runtime_filename, runtime_pkg)])
    return manifest


def bootstrap_external_app(
    *,
    app_name,
    app_version,
    app_message,
    pkg_out,
    repo_out,
    disk_out,
    repo_key,
):
    payload = build_debug_write_app(app_message)
    pkg_blob = build_pkg_v1(app_name, app_version, payload)
    pkg_manifest, _ = parse_pkg_v1(pkg_blob)

    generated_at = datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    metadata = build_repo_metadata_for_pkg(
        pkg_manifest=pkg_manifest,
        pkg_filename=os.path.basename(pkg_out),
        pkg_blob=pkg_blob,
        generated_at=generated_at,
    )
    signed_repo = sign_repo_metadata(metadata, repo_key)
    if not verify_repo_metadata(signed_repo, repo_key):
        raise RuntimeError("internal error: metadata signature verification failed")

    os.makedirs(os.path.dirname(pkg_out) or ".", exist_ok=True)
    os.makedirs(os.path.dirname(repo_out) or ".", exist_ok=True)
    with open(pkg_out, "wb") as f:
        f.write(pkg_blob)
    with open(repo_out, "w", encoding="utf-8") as f:
        json.dump(signed_repo, f, indent=2, sort_keys=True)
        f.write("\n")

    install_pkg_v1_to_simplefs(pkg_blob, disk_out)
    return {
        "pkg_manifest": pkg_manifest,
        "repo_metadata": signed_repo,
        "disk_path": disk_out,
    }


def _parse_args():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app-name", default="hello")
    parser.add_argument("--app-version", default="1.0.0")
    parser.add_argument("--app-message", default="APP: hello world\n")
    parser.add_argument("--pkg-out", default="out/hello.pkgv1")
    parser.add_argument("--repo-out", default="out/repo-v1.json")
    parser.add_argument("--disk-out", default="out/fs-external.img")
    parser.add_argument("--repo-key", default=DEFAULT_REPO_KEY)
    return parser.parse_args()


def main():
    args = _parse_args()
    result = bootstrap_external_app(
        app_name=args.app_name,
        app_version=args.app_version,
        app_message=args.app_message,
        pkg_out=args.pkg_out,
        repo_out=args.repo_out,
        disk_out=args.disk_out,
        repo_key=args.repo_key,
    )

    print(f"==> package v1: {args.pkg_out}")
    print(
        "    name={name} version={version} sha256={sha}".format(
            name=result["pkg_manifest"]["name"],
            version=result["pkg_manifest"]["version"],
            sha=result["pkg_manifest"]["payload_sha256"],
        )
    )
    print(f"==> signed repository metadata: {args.repo_out}")
    print(f"==> install image: {result['disk_path']}")


if __name__ == "__main__":
    main()
