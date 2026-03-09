"""M26 aggregate gate: update trust and key rotation checks."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import check_update_trust_v1 as trust  # noqa: E402
import run_update_key_rotation_drill_v1 as rotation  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_update_trust_gate_v1(tmp_path: Path):
    required = [
        "docs/M26_EXECUTION_BACKLOG.md",
        "docs/pkg/update_trust_model_v1.md",
        "docs/security/update_key_rotation_policy_v1.md",
        "tools/check_update_trust_v1.py",
        "tools/run_update_key_rotation_drill_v1.py",
        "tests/pkg/test_update_trust_docs_v1.py",
        "tests/pkg/test_update_metadata_expiry_v1.py",
        "tests/pkg/test_update_freeze_attack_v1.py",
        "tests/pkg/test_update_mix_and_match_v1.py",
        "tests/pkg/test_update_key_rotation_v1.py",
        "tests/pkg/test_update_trust_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing gate artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    assert "test-update-trust-v1" in roadmap

    assert "test-update-trust-v1" in makefile
    for entry in [
        "tools/check_update_trust_v1.py --out $(OUT)/update-trust-v1.json",
        "tools/run_update_key_rotation_drill_v1.py --out $(OUT)/update-key-rotation-drill-v1.json",
        "tests/pkg/test_update_trust_docs_v1.py",
        "tests/pkg/test_update_metadata_expiry_v1.py",
        "tests/pkg/test_update_freeze_attack_v1.py",
        "tests/pkg/test_update_mix_and_match_v1.py",
        "tests/pkg/test_update_key_rotation_v1.py",
        "tests/pkg/test_update_trust_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-update-trust-v1.xml" in makefile

    assert "Update trust v1 gate" in ci
    assert "make test-update-trust-v1" in ci
    assert "update-trust-v1-artifacts" in ci
    assert "out/pytest-update-trust-v1.xml" in ci
    assert "out/update-trust-v1.json" in ci
    assert "out/update-key-rotation-drill-v1.json" in ci

    trust_out = tmp_path / "update-trust-v1.json"
    rotate_out = tmp_path / "update-key-rotation-drill-v1.json"
    assert trust.main(["--out", str(trust_out), "--max-failures", "0"]) == 0
    assert rotation.main(["--out", str(rotate_out)]) == 0

    trust_data = json.loads(trust_out.read_text(encoding="utf-8"))
    rotate_data = json.loads(rotate_out.read_text(encoding="utf-8"))
    assert trust_data["schema"] == "rugo.update_trust_report.v1"
    assert trust_data["total_failures"] == 0
    assert rotate_data["schema"] == "rugo.update_key_rotation_drill.v1"
    assert rotate_data["success"] is True
