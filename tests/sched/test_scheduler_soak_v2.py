"""M16 acceptance: seeded scheduler soak and regression signal output."""

import json
from pathlib import Path
import sys

import pytest

sys.path.append(str(Path(__file__).resolve().parent))
from sched_v2_model import run_scheduler_soak  # noqa: E402


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _fail_with_report(report: dict, reason: str) -> None:
    payload = {"reason": reason, "report": report}
    pytest.fail(json.dumps(payload, sort_keys=True), pytrace=False)


def test_scheduler_soak_is_deterministic_for_same_seed():
    first = run_scheduler_soak(seed=20260306)
    second = run_scheduler_soak(seed=20260306)

    if first != second:
        _fail_with_report(
            {
                "first": first,
                "second": second,
            },
            "non_deterministic_soak_output",
        )


@pytest.mark.parametrize("seed", [20260306, 20260307, 20260308])
def test_scheduler_soak_has_no_starvation_or_missing_dispatch(seed):
    report = run_scheduler_soak(seed=seed)
    if report["anomalies"]:
        _fail_with_report(report, "soak_anomalies_detected")

    assert report["schema"] == "rugo.scheduler_soak_report.v2"
    assert len(report["timeline_digest"]) == 64
    assert report["ticks"] == 1200


def test_scheduling_policy_v2_doc_declares_soak_schema():
    policy = _read("docs/abi/scheduling_policy_v2.md")
    for token in [
        "Soak report schema: `rugo.scheduler_soak_report.v2`",
        "Failure output must include machine-readable fields:",
        "- `timeline_digest`",
        "- `anomalies`",
    ]:
        assert token in policy
