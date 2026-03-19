"""X4 PR-4: desktop profile runtime aggregate doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_x4_runtime_doc_artifacts_exist():
    required = [
        "docs/desktop/desktop_profile_runtime_v1.md",
        "tools/x4_desktop_runtime_common_v1.py",
        "tools/run_desktop_profile_runtime_v1.py",
        "tests/desktop/test_desktop_profile_runtime_docs_v1.py",
        "tests/desktop/test_desktop_profile_runtime_v1.py",
        "tests/desktop/test_desktop_profile_runtime_gate_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing X4 runtime doc artifact: {rel}"


def test_desktop_profile_runtime_doc_declares_required_tokens():
    doc = _read("docs/desktop/desktop_profile_runtime_v1.md")
    for token in [
        "Qualification report schema: `rugo.desktop_profile_runtime_report.v1`.",
        "Qualification policy ID: `rugo.desktop_profile_runtime.v1`.",
        "Desktop profile ID: `rugo.desktop_profile.v2`.",
        "Runtime capture schema: `rugo.booted_runtime_capture.v1`.",
        "Runtime tool: `tools/run_desktop_profile_runtime_v1.py`.",
        "Boot image: `out/os-go-desktop.iso`.",
        "Kernel image: `out/kernel-go-desktop.elf`.",
        "Primary runtime capture: `out/desktop-profile-capture-v1.json`.",
        "Primary report: `out/desktop-profile-runtime-v1.json`.",
        "Local gate: `make test-desktop-profile-runtime-v1`.",
        "CI gate: `Desktop profile runtime v1 gate`.",
        "CI artifact: `desktop-profile-runtime-v1-artifacts`.",
        "`desktop_bootstrap`",
        "`display_scanout`",
        "`input_seat`",
        "`window_compositor`",
        "`gui_runtime`",
        "`shell_workflows`",
        "`graphical_installer`",
        "`M35`",
        "`M44`",
        "`M48`",
        "`M49`",
        "`M50`",
        "`M51`",
        "`M52`",
    ]:
        assert token in doc


def test_x4_roadmap_docs_record_runtime_backed_closure():
    expansion = _read("docs/roadmap/implementation_closure/expansion_breadth.md")
    summary = _read("docs/roadmap/README.md")
    framework = _read("docs/roadmap/MILESTONE_FRAMEWORK.md")
    gui_roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")

    assert "The historical X4 desktop and workflow backlog is now runtime-backed" in expansion
    for row in [
        "| `M35 Desktop + Interactive UX Baseline v1` | `Runtime-backed` |",
        "| `M44 Real Desktop + Ecosystem Qualification v2` | `Runtime-backed` |",
        "| `M48 Display Runtime + Scanout v1` | `Runtime-backed` |",
        "| `M49 Input + Seat Management v1` | `Runtime-backed` |",
        "| `M50 Window System + Composition v1` | `Runtime-backed` |",
        "| `M51 GUI Runtime + Toolkit Bridge v1` | `Runtime-backed` |",
        "| `M52 Desktop Shell + Workflow Baseline v1` | `Runtime-backed` |",
    ]:
        assert row in expansion

    assert "historical X4 desktop and workflow backlog is closed on a shared runtime-backed qualification lane" in summary
    assert "historical X4 desktop and workflow backlog is closed on a shared runtime-backed qualification lane" in framework
    assert "test-desktop-profile-runtime-v1" in gui_roadmap
