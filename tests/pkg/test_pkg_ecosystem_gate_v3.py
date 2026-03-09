"""M26 aggregate gate: package/repo ecosystem v3 wiring and closure."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import check_update_trust_v1 as trust  # noqa: E402
import pkg_rebuild_verify_v3 as rebuild  # noqa: E402
import repo_policy_check_v3 as policy  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_pkg_ecosystem_gate_v3_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M26_EXECUTION_BACKLOG.md",
        "docs/pkg/package_format_v3.md",
        "docs/pkg/repository_policy_v3.md",
        "docs/pkg/update_trust_model_v1.md",
        "docs/security/update_key_rotation_policy_v1.md",
        "tools/repo_policy_check_v3.py",
        "tools/pkg_rebuild_verify_v3.py",
        "tools/check_update_trust_v1.py",
        "tools/run_update_key_rotation_drill_v1.py",
        "tests/pkg/test_pkg_contract_docs_v3.py",
        "tests/pkg/test_pkg_rebuild_repro_v3.py",
        "tests/pkg/test_repo_policy_v3.py",
        "tests/pkg/test_update_trust_docs_v1.py",
        "tests/pkg/test_update_metadata_expiry_v1.py",
        "tests/pkg/test_update_freeze_attack_v1.py",
        "tests/pkg/test_update_mix_and_match_v1.py",
        "tests/pkg/test_update_key_rotation_v1.py",
        "tests/pkg/test_update_trust_gate_v1.py",
        "tests/pkg/test_pkg_ecosystem_gate_v3.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M26 artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M26_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-pkg-ecosystem-v3" in roadmap
    assert "test-update-trust-v1" in roadmap

    assert "test-pkg-ecosystem-v3" in makefile
    for entry in [
        "tools/repo_policy_check_v3.py --out $(OUT)/repo-policy-v3.json",
        "tools/pkg_rebuild_verify_v3.py --seed 20260309 --out $(OUT)/pkg-rebuild-v3.json",
        "tests/pkg/test_pkg_contract_docs_v3.py",
        "tests/pkg/test_pkg_rebuild_repro_v3.py",
        "tests/pkg/test_repo_policy_v3.py",
        "tests/pkg/test_pkg_ecosystem_gate_v3.py",
        "tools/check_update_trust_v1.py --out $(OUT)/update-trust-v1.json",
        "tools/run_update_key_rotation_drill_v1.py --out $(OUT)/update-key-rotation-drill-v1.json",
        "tests/pkg/test_update_trust_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-pkg-ecosystem-v3.xml" in makefile
    assert "pytest-update-trust-v1.xml" in makefile

    assert "Package ecosystem v3 gate" in ci
    assert "make test-pkg-ecosystem-v3" in ci
    assert "pkg-ecosystem-v3-artifacts" in ci
    assert "out/pytest-pkg-ecosystem-v3.xml" in ci
    assert "out/repo-policy-v3.json" in ci
    assert "out/pkg-rebuild-v3.json" in ci

    assert "Update trust v1 gate" in ci
    assert "make test-update-trust-v1" in ci
    assert "update-trust-v1-artifacts" in ci

    assert "Status: done" in backlog
    assert "M26" in milestones
    assert "M26" in status
    assert "M26" in readme

    policy_out = tmp_path / "repo-policy-v3.json"
    rebuild_out = tmp_path / "pkg-rebuild-v3.json"
    trust_out = tmp_path / "update-trust-v1.json"

    assert policy.main(["--out", str(policy_out), "--max-failures", "0"]) == 0
    assert rebuild.main(["--seed", "20260309", "--out", str(rebuild_out)]) == 0
    assert trust.main(["--out", str(trust_out), "--max-failures", "0"]) == 0

    policy_data = json.loads(policy_out.read_text(encoding="utf-8"))
    rebuild_data = json.loads(rebuild_out.read_text(encoding="utf-8"))
    trust_data = json.loads(trust_out.read_text(encoding="utf-8"))

    assert policy_data["schema"] == "rugo.repo_policy_report.v3"
    assert policy_data["meets_target"] is True
    assert rebuild_data["schema"] == "rugo.pkg_rebuild_report.v3"
    assert rebuild_data["total_mismatches"] == 0
    assert trust_data["schema"] == "rugo.update_trust_report.v1"
    assert trust_data["total_failures"] == 0
