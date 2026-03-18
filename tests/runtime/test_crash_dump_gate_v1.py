"""M29 aggregate gate: crash dump v1 sub-gate wiring and artifacts."""

import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
sys.path.append(str(ROOT / "tools"))

import collect_crash_dump_v1 as collector  # noqa: E402
import symbolize_crash_dump_v1 as symbolizer  # noqa: E402


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_crash_dump_gate_v1_wiring_and_artifacts(tmp_path: Path):
    required = [
        "docs/M29_EXECUTION_BACKLOG.md",
        "docs/runtime/crash_dump_contract_v1.md",
        "docs/runtime/postmortem_triage_playbook_v1.md",
        "tools/collect_crash_dump_v1.py",
        "tools/symbolize_crash_dump_v1.py",
        "tests/runtime/test_crash_dump_docs_v1.py",
        "tests/runtime/test_crash_dump_capture_v1.py",
        "tests/runtime/test_crash_dump_symbolization_v1.py",
        "tests/runtime/test_crash_dump_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing gate artifact: {rel}"

    roadmap = _read("docs/M21_M34_MATURITY_PARITY_ROADMAP.md")
    makefile = _read("Makefile")
    ci = _read(".github/workflows/ci.yml")
    backlog = _read("docs/M29_EXECUTION_BACKLOG.md")
    milestones = _read("MILESTONES.md")
    status = _read("docs/STATUS.md")

    assert "test-crash-dump-v1" in roadmap

    assert "test-crash-dump-v1" in makefile
    for entry in [
        "tools/collect_crash_dump_v1.py --release-image $(OUT)/os-go.iso --kernel $(OUT)/kernel-go.elf --panic-image $(OUT)/os-panic.iso --out $(OUT)/crash-dump-v1.json",
        "tools/symbolize_crash_dump_v1.py --dump $(OUT)/crash-dump-v1.json --out $(OUT)/crash-dump-symbolized-v1.json",
        "tests/runtime/test_crash_dump_docs_v1.py",
        "tests/runtime/test_crash_dump_capture_v1.py",
        "tests/runtime/test_crash_dump_symbolization_v1.py",
        "tests/runtime/test_crash_dump_gate_v1.py",
    ]:
        assert entry in makefile
    assert "pytest-crash-dump-v1.xml" in makefile

    assert "Crash dump v1 gate" in ci
    assert "make test-crash-dump-v1" in ci
    assert "crash-dump-v1-artifacts" in ci
    assert "out/pytest-crash-dump-v1.xml" in ci
    assert "out/crash-dump-v1.json" in ci
    assert "out/crash-dump-symbolized-v1.json" in ci

    assert "Status: done" in backlog
    assert "M29" in milestones
    assert "M29" in status

    dump = tmp_path / "crash-dump-v1.json"
    sym = tmp_path / "crash-dump-symbolized-v1.json"
    assert collector.main(["--fixture", "--out", str(dump)]) == 0
    assert symbolizer.main(["--dump", str(dump), "--out", str(sym)]) == 0
    data = json.loads(sym.read_text(encoding="utf-8"))
    assert data["schema"] == "rugo.crash_dump_symbolized.v1"
    assert data["gate_pass"] is True
    assert len(data["frames"]) >= 1
