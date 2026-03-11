"""M48 PR-1: display runtime contract doc checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m48_pr1_display_runtime_artifacts_exist():
    required = [
        "docs/M48_EXECUTION_BACKLOG.md",
        "docs/desktop/display_runtime_contract_v1.md",
        "docs/desktop/scanout_buffer_contract_v1.md",
        "docs/desktop/gpu_fallback_policy_v1.md",
        "tests/desktop/test_display_runtime_docs_v1.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M48 PR-1 artifact: {rel}"


def test_display_runtime_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/desktop/display_runtime_contract_v1.md")
    for token in [
        "Display runtime contract ID: `rugo.display_runtime_contract.v1`.",
        "Parent display stack contract ID: `rugo.display_stack_contract.v1`.",
        "Runtime report schema: `rugo.display_runtime_report.v1`.",
        "Frame capture schema: `rugo.display_frame_capture.v1`.",
        "`virtio-gpu-pci`",
        "`virtio_gpu_scanout`",
        "`framebuffer-console`",
        "`efifb`",
        "`present_timing_budget`",
        "`frame_capture_ready`",
        "Local gate: `make test-display-runtime-v1`.",
        "Local sub-gate: `make test-scanout-path-v1`.",
        "CI gate: `Display runtime v1 gate`.",
        "CI sub-gate: `Scanout path v1 gate`.",
    ]:
        assert token in doc


def test_scanout_buffer_contract_v1_doc_declares_required_tokens():
    doc = _read("docs/desktop/scanout_buffer_contract_v1.md")
    for token in [
        "Buffer contract ID: `rugo.scanout_buffer_contract.v1`.",
        "Parent runtime contract ID: `rugo.display_runtime_contract.v1`.",
        "Runtime report schema: `rugo.display_runtime_report.v1`.",
        "Frame capture schema: `rugo.display_frame_capture.v1`.",
        "Minimum scanout buffers: `3`.",
        "Required pixel format: `xrgb8888`.",
        "`runtime_owned`",
        "`scanout_pending`",
        "`display_owned`",
        "`capture_read_only`",
        "`buffer_ownership_integrity`",
        "`scanout_buffer_depth`",
    ]:
        assert token in doc


def test_gpu_fallback_policy_v1_doc_declares_required_tokens():
    doc = _read("docs/desktop/gpu_fallback_policy_v1.md")
    for token in [
        "Fallback policy ID: `rugo.gpu_fallback_policy.v1`.",
        "Parent runtime contract ID: `rugo.display_runtime_contract.v1`.",
        "Runtime report schema: `rugo.display_runtime_report.v1`.",
        "Prefer `virtio-gpu-pci` with runtime driver `virtio_gpu_scanout`.",
        "Permit `force_fallback` to select `framebuffer-console` with `efifb`.",
        "`efifb_fallback_activation` must remain `<= 80 ms`.",
        "`efifb_fallback_scanout` must remain `<= 0.01` frame-drop ratio.",
        "`primary`",
        "`forced_fallback`",
        "`auto_fallback`",
        "Local gate: `make test-display-runtime-v1`.",
        "Local sub-gate: `make test-scanout-path-v1`.",
    ]:
        assert token in doc


def test_m48_roadmap_anchor_declares_display_runtime_gates():
    roadmap = _read("docs/M48_M52_GUI_IMPLEMENTATION_ROADMAP.md")
    assert "test-display-runtime-v1" in roadmap
    assert "test-scanout-path-v1" in roadmap
