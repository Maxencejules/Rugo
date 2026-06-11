# Phase 10c acceptance: users and permissions. The shell (root, uid 0)
# creates a file with the default mode (owner rw, other r). The fsperm
# probe runs as uid 100: write and unlink must be DENIED, read allowed.
# After a root chmod to 15 (other rw) the same probe succeeds at all
# three - including unlinking the file, which a follow-up cat proves.


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


def test_unprivileged_app_is_gated_by_file_mode(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot(
        "fsmk /data/etc\n"
        "fswrite /data/etc/secret root-only-content\n"
        "fsperm /data/etc/secret\n"
        "fschmod /data/etc/secret 15\n"
        "fsperm /data/etc/secret\n"
        "fscat /data/etc/secret\n"
        "shutdown\n"
    ).stdout

    _find_in_order(out, [
        "FSH: write ok",
        # uid 100 vs default mode: write/unlink denied, read allowed
        "FSPERM: write denied",
        "FSPERM: read ok",
        "FSPERM: unlink denied",
        "FSH: chmod ok",
        # uid 100 vs other-rw: everything allowed, file unlinked
        "FSPERM: write ok",
        "FSPERM: read ok",
        "FSPERM: unlink ok",
        # the file is gone
        "FSH: err",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert out.count("FSPERM: write denied") == 1
    assert out.count("FSPERM: unlink denied") == 1
    assert out.count("FSPERM: unlink ok") == 1
    assert "FSPERM: read denied" not in out
    assert "GOINIT: err" not in out
