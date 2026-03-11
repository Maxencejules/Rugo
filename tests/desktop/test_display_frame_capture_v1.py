"""M48 PR-2: deterministic display frame capture checks."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import capture_display_frame_v1 as capture  # noqa: E402


PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def _out_path(name: str) -> Path:
    path = ROOT / "out" / "pytest-m48" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        path.unlink()
    sidecar = path.with_suffix(".json")
    if sidecar.exists():
        sidecar.unlink()
    return path


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    stable.pop("artifact_refs", None)
    return stable


def test_display_frame_capture_v1_deterministic_png_and_manifest():
    first_png = _out_path("display-frame-v1-a.png")
    second_png = _out_path("display-frame-v1-b.png")

    assert capture.main(["--seed", "20260311", "--out", str(first_png)]) == 0
    assert capture.main(["--seed", "20260311", "--out", str(second_png)]) == 0

    first_bytes = first_png.read_bytes()
    second_bytes = second_png.read_bytes()
    assert first_bytes == second_bytes

    first_manifest = json.loads(first_png.with_suffix(".json").read_text(encoding="utf-8"))
    second_manifest = json.loads(second_png.with_suffix(".json").read_text(encoding="utf-8"))
    assert _strip_timestamp(first_manifest) == _strip_timestamp(second_manifest)


def test_display_frame_capture_v1_schema_and_png_signature():
    out = _out_path("display-frame-v1.png")
    rc = capture.main(["--seed", "20260311", "--out", str(out)])
    assert rc == 0

    png_bytes = out.read_bytes()
    manifest = json.loads(out.with_suffix(".json").read_text(encoding="utf-8"))

    assert png_bytes.startswith(PNG_SIGNATURE)
    assert manifest["schema"] == "rugo.display_frame_capture.v1"
    assert manifest["active_runtime_path"] == "virtio-gpu-pci"
    assert manifest["width"] == 320
    assert manifest["height"] == 180
    assert manifest["capture_pass"] is True
    assert manifest["gate_pass"] is True
    assert manifest["png_sha256"] == hashlib.sha256(png_bytes).hexdigest()


def test_display_frame_capture_v1_detects_capture_regression():
    out = _out_path("display-frame-v1-fail.png")
    rc = capture.main(
        [
            "--inject-runtime-failure",
            "frame_capture_ready",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    manifest = json.loads(out.with_suffix(".json").read_text(encoding="utf-8"))
    assert manifest["schema"] == "rugo.display_frame_capture.v1"
    assert manifest["capture_pass"] is False
    assert manifest["gate_pass"] is False
