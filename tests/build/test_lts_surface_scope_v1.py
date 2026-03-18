"""M34 acceptance: LTS scope is explicitly bounded to the shipped surface."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import run_maturity_qualification_v1 as maturity  # noqa: E402


def test_lts_surface_scope_v1_is_bounded_to_default_lane(tmp_path: Path):
    out = tmp_path / "maturity-qualification-v1.json"
    assert maturity.main(["--fixture", "--out", str(out)]) == 0

    data = json.loads(out.read_text(encoding="utf-8"))
    qualified_surface = data["qualified_surface"]
    lts_surface = data["lts_declaration"]["supported_surface"]

    assert qualified_surface["execution_lane"] == "qemu"
    assert qualified_surface["supported_profiles"] == ["server_v1", "appliance_v1"]
    assert qualified_surface["non_lts_profiles"] == ["developer_v1"]
    assert qualified_surface["support_matrix"][0]["target_id"] == "qemu-q35-default-lane"
    assert lts_surface["supported_profiles"] == ["server_v1", "appliance_v1"]
    criteria = {entry["name"]: entry["pass"] for entry in data["lts_declaration"]["criteria"]}
    assert criteria["default_lane_target_scoped"] is True
    assert criteria["lts_profiles_qualified"] is True
    assert criteria["performance_budget_pass"] is True
