"""M23 acceptance: TPM event log schema invariants."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_measured_boot_report_v1 as tool  # noqa: E402


def test_tpm_eventlog_schema(tmp_path: Path):
    out = tmp_path / "measured-boot-v1.json"
    rc = tool.main(["--out", str(out)])
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    events = data["tpm_event_log"]
    assert data["event_count"] == len(events)
    assert len(events) >= 4
    for entry in events:
        assert "index" in entry
        assert "pcr" in entry
        assert "type" in entry
        assert "component" in entry
        assert "digest" in entry
        assert len(entry["digest"]) == 64
