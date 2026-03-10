"""M41 PR-1: compatibility profile v5 process/readiness doc contract checks."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def test_m41_pr1_compat_contract_artifacts_exist():
    required = [
        "docs/M41_EXECUTION_BACKLOG.md",
        "docs/abi/compat_profile_v5.md",
        "docs/runtime/syscall_coverage_matrix_v4.md",
        "docs/abi/process_model_v4.md",
        "docs/abi/readiness_io_model_v1.md",
        "tests/compat/test_compat_docs_v5.py",
    ]
    for rel in required:
        assert (ROOT / rel).is_file(), f"missing M41 PR-1 artifact: {rel}"


def test_compat_profile_v5_doc_declares_required_tokens():
    doc = _read("docs/abi/compat_profile_v5.md")
    for token in [
        "Compatibility profile identifier: `rugo.compat_profile.v5`.",
        "Process model contract ID: `rugo.process_model.v4`.",
        "Readiness I/O contract ID: `rugo.readiness_io_model.v1`.",
        "POSIX gap report schema: `rugo.posix_gap_report.v2`.",
        "Compatibility surface campaign schema: `rugo.compat_surface_campaign_report.v2`.",
        "Local gate: `make test-process-readiness-parity-v1`.",
        "Local sub-gate: `make test-posix-gap-closure-v2`.",
        "CI gate: `Process readiness parity v1 gate`.",
        "CI sub-gate: `POSIX gap closure v2 gate`.",
    ]:
        assert token in doc


def test_syscall_coverage_matrix_v4_doc_declares_required_tokens():
    doc = _read("docs/runtime/syscall_coverage_matrix_v4.md")
    for token in [
        "Milestone: M41 Process + Readiness Compatibility Closure v1",
        "`fork`",
        "`clone`",
        "`epoll`",
        "`io_uring`",
        "`poll`",
        "`eventfd`",
        "process/readiness gate | `test-process-readiness-parity-v1`",
        "posix sub-gate | `test-posix-gap-closure-v2`",
    ]:
        assert token in doc


def test_process_model_v4_doc_declares_required_tokens():
    doc = _read("docs/abi/process_model_v4.md")
    for token in [
        "Process model contract ID: `rugo.process_model.v4`",
        "Parent compatibility profile ID: `rugo.compat_profile.v5`",
        "Surface campaign schema: `rugo.compat_surface_campaign_report.v2`",
        "`process_spawn_exec`",
        "`process_wait_reap_once`",
        "`process_signal_fifo`",
        "`process_sigkill_terminal`",
        "`process_pid_reuse_guard`",
        "spawn-to-ready latency: `<= 130 ms`",
        "wait/reap latency: `<= 22 ms`",
    ]:
        assert token in doc


def test_readiness_io_model_v1_doc_declares_required_tokens():
    doc = _read("docs/abi/readiness_io_model_v1.md")
    for token in [
        "Readiness I/O model contract ID: `rugo.readiness_io_model.v1`",
        "Parent compatibility profile ID: `rugo.compat_profile.v5`",
        "Surface campaign schema: `rugo.compat_surface_campaign_report.v2`",
        "POSIX gap report schema: `rugo.posix_gap_report.v2`",
        "`readiness_poll_wakeup`",
        "`readiness_ppoll_wakeup`",
        "`readiness_pselect_wakeup`",
        "`deferred_epoll_enosys`",
        "`deferred_io_uring_enosys`",
        "poll wakeup latency: `<= 11 ms`",
    ]:
        assert token in doc


def test_m40_m44_roadmap_anchor_declares_m41_gates():
    roadmap = _read("docs/M40_M44_GENERAL_PURPOSE_PARITY_ROADMAP.md")
    assert "test-process-readiness-parity-v1" in roadmap
    assert "test-posix-gap-closure-v2" in roadmap
