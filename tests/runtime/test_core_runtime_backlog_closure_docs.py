"""Roadmap docs: historical core-runtime backlog closure wording stays aligned."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_core_runtime_backlog_closure_doc_marks_historical_backlog_closed():
    doc = _read("docs/roadmap/implementation_closure/core_runtime.md")

    for token in [
        "The backlog itself is closed in the repo ledger",
        "## Historical Closure Sequence",
        "| Backlog | Backlog state | Implementation class | Closure basis in repo | Carry-forward product work |",
        "| `M10 Security baseline v1` | `Closed` | `Runtime-backed` |",
        "| `M16 Process + Scheduler Model v2` | `Closed` | `Runtime-backed` |",
        "| `M25 Userspace Service Model + Init v2` | `Closed` | `Runtime-backed` |",
        "| `M18 Storage reliability v2` | `Closed` | `Runtime-backed` |",
        "| `M19 Network stack v2` | `Closed` | `Runtime-backed` |",
        "| `M22 Kernel Reliability + Soak v1` | `Closed` | `Runtime-backed` |",
        "| `M42 Isolation + Namespace Baseline v1` | `Closed` | `Runtime-backed` |",
        "Treat `M22` and `M42` as closed historical",
    ]:
        assert token in doc


def test_roadmap_summary_docs_use_historical_core_backlog_language():
    for relpath in [
        "docs/roadmap/README.md",
        "docs/roadmap/MILESTONE_FRAMEWORK.md",
    ]:
        doc = _read(relpath)
        assert "Historical Core Backlog Order" in doc
        assert "historical core-runtime backlog is closed in the ledger" in doc
