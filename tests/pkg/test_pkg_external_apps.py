"""M8 PR-3: external package bootstrap and run lane."""

import hashlib
import json
import struct
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(REPO_ROOT / "tools"))

import pkg_bootstrap_v1 as pkgv1


def test_pkg_v1_manifest_roundtrip():
    payload = pkgv1.build_debug_write_app("APP: external lane\n")
    blob = pkgv1.build_pkg_v1("external-hello", "1.2.3", payload)
    manifest, parsed_payload = pkgv1.parse_pkg_v1(blob)

    assert manifest["schema"] == "rugo.pkg.v1"
    assert manifest["name"] == "external-hello"
    assert manifest["version"] == "1.2.3"
    assert manifest["payload_size"] == len(payload)
    assert manifest["payload_sha256"] == hashlib.sha256(payload).hexdigest()
    assert parsed_payload == payload


def test_repo_metadata_signature_verification():
    payload = pkgv1.build_debug_write_app("APP: external lane\n")
    blob = pkgv1.build_pkg_v1("external-hello", "1.2.3", payload)
    manifest, _ = pkgv1.parse_pkg_v1(blob)

    metadata = pkgv1.build_repo_metadata_for_pkg(
        pkg_manifest=manifest,
        pkg_filename="external-hello.pkgv1",
        pkg_blob=blob,
        generated_at="2026-03-04T00:00:00+00:00",
    )
    signed = pkgv1.sign_repo_metadata(metadata, "test-key")
    assert pkgv1.verify_repo_metadata(signed, "test-key") is True

    tampered = json.loads(json.dumps(signed))
    tampered["metadata"]["packages"][0]["pkg_size"] += 1
    assert pkgv1.verify_repo_metadata(tampered, "test-key") is False


def test_install_bridge_writes_runtime_hello_pkg(tmp_path):
    payload = pkgv1.build_debug_write_app("APP: bridge install\n")
    blob = pkgv1.build_pkg_v1("external-hello", "1.2.3", payload)
    disk_path = tmp_path / "fs-external.img"

    manifest = pkgv1.install_pkg_v1_to_simplefs(blob, str(disk_path))
    assert manifest["name"] == "external-hello"
    assert disk_path.is_file()
    assert disk_path.stat().st_size == 1024 * 1024

    raw = disk_path.read_bytes()
    sb_magic, file_count, data_start, _next_free = struct.unpack("<IIII", raw[:16])
    assert sb_magic == pkgv1.SFS_MAGIC
    assert file_count == 1
    assert data_start == 2

    entry = raw[512 : 512 + 32]
    name = entry[:24].split(b"\x00", 1)[0]
    start_sector, size_bytes = struct.unpack("<II", entry[24:32])
    assert name == b"hello.pkg"
    assert start_sector >= 2
    assert size_bytes > 64

    pkg_offset = start_sector * 512
    pkg_magic = struct.unpack("<I", raw[pkg_offset : pkg_offset + 4])[0]
    assert pkg_magic == pkgv1.PKG_V0_MAGIC


def test_external_package_runs_in_qemu(qemu_serial_pkg_external):
    out = qemu_serial_pkg_external.stdout
    assert "PKG: hash ok" in out, f"Missing package hash marker. Got:\n{out}"
    assert "APP: hello world" in out, f"Missing external app marker. Got:\n{out}"
