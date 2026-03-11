"""M50 PR-1: window system contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m50_pr1_window_system_artifacts_exist():
    required = [
        "docs/M50_EXECUTION_BACKLOG.md",
        "docs/desktop/surface_lifecycle_contract_v1.md",
        "docs/desktop/compositor_damage_policy_v1.md",
        "docs/desktop/window_manager_contract_v2.md",
        "tests/desktop/test_window_system_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M50 PR-1 artifact: {rel}"


def test_surface_lifecycle_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/desktop/surface_lifecycle_contract_v1.md")
    for token in [
        "Surface lifecycle contract ID: `rugo.surface_lifecycle_contract.v1`.",
        "Parent display runtime contract ID: `rugo.display_runtime_contract.v1`.",
        "Parent seat contract ID: `rugo.seat_input_contract.v1`.",
        "Runtime report schema: `rugo.window_system_runtime_report.v1`.",
        "Compositor damage schema: `rugo.compositor_damage_report.v1`.",
        "`created`",
        "`mapped`",
        "`visible`",
        "`occluded`",
        "`unmapped`",
        "`destroyed`",
        "`surface_release_budget`",
        "Local gate: `make test-window-system-v1`.",
        "Local sub-gate: `make test-compositor-damage-v1`.",
        "CI gate: `Window system v1 gate`.",
        "CI sub-gate: `Compositor damage v1 gate`.",
    ]:
        assert token in doc


def test_compositor_damage_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/desktop/compositor_damage_policy_v1.md")
    for token in [
        "Compositor damage policy ID: `rugo.compositor_damage_policy.v1`.",
        "Parent surface lifecycle contract ID: `rugo.surface_lifecycle_contract.v1`.",
        "Parent display runtime contract ID: `rugo.display_runtime_contract.v1`.",
        "Runtime report schema: `rugo.window_system_runtime_report.v1`.",
        "Compositor damage schema: `rugo.compositor_damage_report.v1`.",
        "Damage union policy: `bounding_union_per_output`.",
        "Opaque clip policy: `front_to_back_opaque_clip`.",
        "`damage_region_union`",
        "`occlusion_clip_integrity`",
        "`present_region_budget`",
        "`fullscreen_damage_reset`",
    ]:
        assert token in doc


def test_window_manager_contract_v2_doc_declares_required_tokens():
    doc = _read("docs/desktop/window_manager_contract_v2.md")
    for token in [
        "Window manager contract ID: `rugo.window_manager_contract.v2`.",
        "Supersedes window manager contract ID: `rugo.window_manager_contract.v1`.",
        "Parent desktop profile ID: `rugo.desktop_profile.v2`.",
        "Parent seat contract ID: `rugo.seat_input_contract.v1`.",
        "Runtime report schema: `rugo.window_system_runtime_report.v1`.",
        "Compositor damage schema: `rugo.compositor_damage_report.v1`.",
        "`background`",
        "`normal`",
        "`overlay`",
        "Only the topmost visible `normal` window may own focus.",
        "`z_order_integrity` must remain `0` ordering violations.",
        "`window_move_budget` must remain `<= 24 ms`.",
        "`window_resize_budget` must remain `<= 32 ms`.",
    ]:
        assert token in doc


def test_m50_roadmap_anchor_declares_window_system_gates():
    roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")
    assert "test-window-system-v1" in roadmap
    assert "test-compositor-damage-v1" in roadmap
