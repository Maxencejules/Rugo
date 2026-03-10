"""M44 aggregate gate: real desktop/ecosystem v2 wiring and closure checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_real_catalog_audit_v2 as audit  # noqa: E402
import run_real_gui_app_matrix_v2 as gui  # noqa: E402
import run_real_pkg_install_campaign_v2 as install  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_real_desktop_gate_v2_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M44_EXECUTION_BACKLOG.md",
        "docs/desktop/desktop_profile_v2.md",
        "docs/abi/app_compat_tiers_v2.md",
        "docs/pkg/ecosystem_scale_policy_v2.md",
        "docs/pkg/distribution_workflow_v2.md",
        "tools/run_real_gui_app_matrix_v2.py",
        "tools/run_real_pkg_install_campaign_v2.py",
        "tools/run_real_catalog_audit_v2.py",
        "tests/desktop/test_desktop_docs_v2.py",
        "tests/pkg/test_ecosystem_scale_docs_v2.py",
        "tests/desktop/test_gui_runtime_qualification_v2.py",
        "tests/pkg/test_pkg_install_success_rate_v2.py",
        "tests/pkg/test_catalog_reproducibility_v2.py",
        "tests/pkg/test_distribution_workflow_v2.py",
        "tests/pkg/test_real_catalog_gate_v2.py",
        "tests/desktop/test_real_desktop_gate_v2.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M44 artifact: {rel}"

    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M44_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")
    readme = _read("README.md")

    assert "test-real-ecosystem-desktop-v2" in roadmap
    assert "test-real-app-catalog-v2" in roadmap

    assert "test-real-ecosystem-desktop-v2" in makefile
    for entry in [
        "tools/run_real_gui_app_matrix_v2.py --out $(OUT)/real-gui-matrix-v2.json",
        "$(MAKE) test-real-app-catalog-v2",
        "tests/desktop/test_desktop_docs_v2.py",
        "tests/pkg/test_ecosystem_scale_docs_v2.py",
        "tests/desktop/test_gui_runtime_qualification_v2.py",
        "tests/desktop/test_real_desktop_gate_v2.py",
    ]:
        assert entry in makefile
    assert "pytest-real-ecosystem-desktop-v2.xml" in makefile
    assert "pytest-real-app-catalog-v2.xml" in makefile

    assert "Real ecosystem desktop v2 gate" in ci
    assert "make test-real-ecosystem-desktop-v2" in ci
    assert "real-ecosystem-desktop-v2-artifacts" in ci
    assert "out/pytest-real-ecosystem-desktop-v2.xml" in ci
    assert "out/real-gui-matrix-v2.json" in ci

    assert "Real app catalog v2 gate" in ci
    assert "make test-real-app-catalog-v2" in ci
    assert "real-app-catalog-v2-artifacts" in ci
    assert "out/pytest-real-app-catalog-v2.xml" in ci
    assert "out/real-pkg-install-v2.json" in ci
    assert "out/real-catalog-audit-v2.json" in ci

    assert "Status: done" in backlog
    assert "| M44 | Real Desktop + Ecosystem Qualification v2 | n/a | done |" in milestones
    assert "| **M44** Real Desktop + Ecosystem Qualification v2 | n/a | done |" in status
    assert "M0-M44: done" in readme
    assert "M44 execution backlog (completed)" in readme

    gui_out = tmp_path / "real-gui-matrix-v2.json"
    install_out = tmp_path / "real-pkg-install-v2.json"
    audit_out = tmp_path / "real-catalog-audit-v2.json"

    assert gui.main(["--seed", "20260310", "--out", str(gui_out)]) == 0
    assert install.main(["--seed", "20260310", "--out", str(install_out)]) == 0
    assert audit.main(["--seed", "20260310", "--out", str(audit_out)]) == 0

    gui_data = json.loads(gui_out.read_text(encoding="utf-8"))
    install_data = json.loads(install_out.read_text(encoding="utf-8"))
    audit_data = json.loads(audit_out.read_text(encoding="utf-8"))

    assert gui_data["schema"] == "rugo.real_gui_app_matrix_report.v2"
    assert gui_data["profile_id"] == "rugo.desktop_profile.v2"
    assert gui_data["gate_pass"] is True
    assert gui_data["total_failures"] == 0

    assert install_data["schema"] == "rugo.real_pkg_install_campaign_report.v2"
    assert install_data["ecosystem_policy_id"] == "rugo.ecosystem_scale_policy.v2"
    assert install_data["gate_pass"] is True
    assert install_data["total_failures"] == 0

    assert audit_data["schema"] == "rugo.real_catalog_audit_report.v2"
    assert audit_data["distribution_workflow_id"] == "rugo.distribution_workflow.v2"
    assert audit_data["gate_pass"] is True
    assert audit_data["total_failures"] == 0
