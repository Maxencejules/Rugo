"""M44 aggregate sub-gate: real app catalog v2 wiring and artifacts."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_real_catalog_audit_v2 as audit  # noqa: E402
import run_real_pkg_install_campaign_v2 as install  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_real_catalog_gate_v2_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M44_EXECUTION_BACKLOG.md",
        "docs/pkg/ecosystem_scale_policy_v2.md",
        "docs/pkg/distribution_workflow_v2.md",
        "tools/run_real_pkg_install_campaign_v2.py",
        "tools/run_real_catalog_audit_v2.py",
        "tests/pkg/test_ecosystem_scale_docs_v2.py",
        "tests/pkg/test_pkg_install_success_rate_v2.py",
        "tests/pkg/test_catalog_reproducibility_v2.py",
        "tests/pkg/test_distribution_workflow_v2.py",
        "tests/pkg/test_real_catalog_gate_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M44 app-catalog artifact: {rel}"

    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M44_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-real-app-catalog-v2" in roadmap

    assert "test-real-app-catalog-v2" in makefile
    for entry in [
        "tools/run_real_pkg_install_campaign_v2.py --out $(OUT)/real-pkg-install-v2.json",
        "tools/run_real_catalog_audit_v2.py --out $(OUT)/real-catalog-audit-v2.json",
        "tests/pkg/test_ecosystem_scale_docs_v2.py",
        "tests/pkg/test_pkg_install_success_rate_v2.py",
        "tests/pkg/test_catalog_reproducibility_v2.py",
        "tests/pkg/test_distribution_workflow_v2.py",
        "tests/pkg/test_real_catalog_gate_v2.py",
    ]:
        assert entry in makefile
    assert "pytest-real-app-catalog-v2.xml" in makefile

    assert "Real app catalog v2 gate" in ci
    assert "make test-real-app-catalog-v2" in ci
    assert "real-app-catalog-v2-artifacts" in ci
    assert "out/pytest-real-app-catalog-v2.xml" in ci
    assert "out/real-pkg-install-v2.json" in ci
    assert "out/real-catalog-audit-v2.json" in ci

    assert "Status: done" in backlog
    assert "M44" in milestones
    assert "M44" in status
    assert "M44 execution backlog (completed)" in readme

    install_out = tmp_path / "real-pkg-install-v2.json"
    audit_out = tmp_path / "real-catalog-audit-v2.json"

    assert install.main(["--seed", "20260310", "--out", str(install_out)]) == 0
    assert audit.main(["--seed", "20260310", "--out", str(audit_out)]) == 0

    install_data = json.loads(install_out.read_text(encoding="utf-8"))
    audit_data = json.loads(audit_out.read_text(encoding="utf-8"))

    assert install_data["schema"] == "rugo.real_pkg_install_campaign_report.v2"
    assert install_data["distribution_workflow_id"] == "rugo.distribution_workflow.v2"
    assert install_data["gate_pass"] is True
    assert install_data["total_failures"] == 0

    assert audit_data["schema"] == "rugo.real_catalog_audit_report.v2"
    assert audit_data["distribution_workflow_id"] == "rugo.distribution_workflow.v2"
    assert audit_data["gate_pass"] is True
    assert audit_data["total_failures"] == 0
