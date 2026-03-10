"""M44 PR-2: runtime-qualified distribution workflow checks."""

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


def test_distribution_workflow_v2_doc_declares_required_tokens():
    doc = _read("docs/pkg/distribution_workflow_v2.md")
    for token in [
        "Policy ID: `rugo.distribution_workflow.v2`.",
        "Workflow report schema: `rugo.real_catalog_audit_report.v2`.",
        "Install report schema: `rugo.real_pkg_install_campaign_report.v2`.",
        "`ingest`",
        "`vet`",
        "`sign`",
        "`runtime_qualify`",
        "`stage`",
        "`rollout`",
        "`rollback`",
        "Workflow stage completeness ratio: `>= 1.0`.",
        "Release signoff ratio: `>= 1.0`.",
        "Rollback drill pass ratio: `>= 1.0`.",
        "Mirror index consistency ratio: `>= 1.0`.",
        "Replication lag p95 minutes: `<= 10`.",
        "Runtime trace coverage ratio: `>= 1.0`.",
        "Signed artifact ratio: `>= 1.0`.",
    ]:
        assert token in doc


def test_distribution_workflow_v2_artifacts_pass(tmp_path: Path):
    install_out = tmp_path / "real-pkg-install-v2.json"
    audit_out = tmp_path / "real-catalog-audit-v2.json"

    assert install.main(["--seed", "20260310", "--out", str(install_out)]) == 0
    assert audit.main(["--seed", "20260310", "--out", str(audit_out)]) == 0

    install_data = json.loads(install_out.read_text(encoding="utf-8"))
    audit_data = json.loads(audit_out.read_text(encoding="utf-8"))

    assert install_data["distribution_workflow_id"] == "rugo.distribution_workflow.v2"
    assert install_data["gate_pass"] is True
    assert install_data["total_failures"] == 0

    assert audit_data["distribution_workflow_id"] == "rugo.distribution_workflow.v2"
    assert audit_data["gate_pass"] is True
    assert audit_data["total_failures"] == 0
    assert audit_data["summary"]["workflow"]["pass"] is True


def test_distribution_workflow_v2_detects_workflow_stage_failure(tmp_path: Path):
    out = tmp_path / "real-catalog-audit-v2-workflow-fail.json"
    rc = audit.main(
        [
            "--inject-failure",
            "workflow_stage_completeness_ratio",
            "--out",
            str(out),
        ]
    )
    assert rc == 1

    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["gate_pass"] is False
    assert data["summary"]["workflow"]["failures"] >= 1
