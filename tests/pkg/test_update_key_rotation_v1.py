"""M26 acceptance: key rotation drill succeeds."""

import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_update_key_rotation_drill_v1 as drill  # noqa: E402


def test_update_key_rotation_drill(tmp_path: Path):
    out = tmp_path / "update-key-rotation-drill-v1.json"
    rc = drill.main(["--out", str(out)])
    assert rc == 0
    data = json.loads(out.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.update_key_rotation_drill.v1"
    assert data["success"] is True
    assert len(data["stages"]) >= 5

    stage_names = [stage["name"] for stage in data["stages"]]
    for required in [
        "old_key_only",
        "overlap_window",
        "new_key_primary",
        "old_key_revoked",
        "revocation_enforced",
    ]:
        assert required in stage_names
