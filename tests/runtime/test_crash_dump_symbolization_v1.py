"""M29 PR-2: deterministic crash dump symbolization checks."""

from __future__ import annotations

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_crash_dump_v1 as collector  # noqa: E402
import symbolize_crash_dump_v1 as symbolizer  # noqa: E402


def _strip_timestamp(payload: dict) -> dict:
    stable = dict(payload)
    stable.pop("created_utc", None)
    return stable


def test_crash_dump_symbolization_is_deterministic_except_timestamp():
    dump = collector.build_dump(
        panic_code=13,
        release_image_digest="digest",
        panic_trace_digest="trace",
    )
    first = symbolizer.symbolize(dump)
    second = symbolizer.symbolize(dump)
    assert _strip_timestamp(first) == _strip_timestamp(second)


def test_crash_dump_symbolization_v1_schema(tmp_path: Path):
    dump = tmp_path / "crash-dump-v1.json"
    sym = tmp_path / "crash-dump-symbolized-v1.json"
    assert collector.main(["--fixture", "--out", str(dump)]) == 0
    assert symbolizer.main(["--dump", str(dump), "--out", str(sym)]) == 0
    data = json.loads(sym.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.crash_dump_symbolized.v1"
    assert data["source_schema"] == "rugo.crash_dump.v1"
    assert data["contract_id"] == "rugo.crash_dump_contract.v1"
    assert data["symbol_map_id"] == "rugo.kernel_symbol_map.v1"
    assert data["triage_playbook_id"] == "rugo.postmortem_triage_playbook.v1"
    assert data["runtime_provenance"]["release_image_path"] == "out/os-go.iso"
    assert data["resolved_frames"] >= 1
    assert data["unresolved_frames"] == 0
    assert data["all_frames_symbolized"] is True
    assert data["gate_pass"] is True
    assert len(data["frames"]) >= 1
    assert "symbol" in data["frames"][0]


def test_crash_dump_symbolization_detects_unresolved_frame(tmp_path: Path):
    dump = tmp_path / "crash-dump-v1.json"
    sym = tmp_path / "crash-dump-symbolized-v1.json"
    assert collector.main(["--fixture", "--out", str(dump)]) == 0

    dump_data = json.loads(dump.read_text(encoding="utf-8"))
    dump_data["stack_frames"].append({"ip": "0xffffffff8badf00d", "offset": 12})
    dump.write_text(json.dumps(dump_data, indent=2) + "\n", encoding="utf-8")

    rc = symbolizer.main(
        [
            "--dump",
            str(dump),
            "--max-unresolved",
            "0",
            "--out",
            str(sym),
        ]
    )
    assert rc == 1
    data = json.loads(sym.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.crash_dump_symbolized.v1"
    assert data["unresolved_frames"] >= 1
    assert data["gate_pass"] is False
