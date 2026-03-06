"""M18 acceptance: storage power-fail campaign v2 report contract."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]

sys.path.append(str(ROOT / "tools"))
import run_storage_powerfail_campaign_v2 as campaign  # noqa: E402


def test_storage_powerfail_campaign_report_schema_and_threshold():
    max_failures = 0
    data = campaign.run_campaign(seed=20260304, iterations=900)
    data["max_failures"] = max_failures
    data["meets_target"] = data["total_failures"] <= max_failures

    assert data["schema"] == "rugo.storage_powerfail_campaign_report.v2"
    assert data["iterations"] == 900
    assert data["total_failures"] == 0
    assert data["meets_target"] is True
    assert data["recovered_cases"] == data["injected_faults"]
