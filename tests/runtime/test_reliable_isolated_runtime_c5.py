"""C5 runtime acceptance: reliability and isolation are exercised on image-go."""


def _has_task_line(serial: str, service: str, tokens: list[str]) -> bool:
    prefix = f"TASK: {service} "
    for line in serial.splitlines():
        if prefix not in line:
            continue
        if all(token in line for token in tokens):
            return True
    return False


def test_reliable_isolated_runtime_c5_runs_boot_backed_isolation_and_soak(
    qemu_go_c4_runtime,
    find_in_order,
):
    boot, _disk_path = qemu_go_c4_runtime

    first = boot().stdout
    find_in_order(
        first,
        [
            "GOSH: diag ok",
            "NETC4: reply ok",
            "ISOC5: domain ok",
            "ISOC5: quota ok",
            "ISOC5: observe ok",
            "SOAKC5: mixed ok",
            "ISOC5: cleanup ok",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )
    assert first.count("DIAGSVC: snapshot") >= 2
    assert _has_task_line(first, "timesvc", ["dom=1", "cap=0", "fd=0", "sock=0"])
    assert _has_task_line(first, "diagsvc", ["dom=2", "cap=0", "fd=0", "sock=0"])
    assert _has_task_line(first, "shell", ["dom=3", "cap=3", "fd=0", "sock=0"])

    second = boot().stdout
    find_in_order(
        second,
        [
            "RECOV: replay ok",
            "STORC4: state ok",
            "STORC4: fsync ok",
            "NETC4: reply ok",
            "ISOC5: domain ok",
            "ISOC5: quota ok",
            "ISOC5: observe ok",
            "SOAKC5: mixed ok",
            "ISOC5: cleanup ok",
            "GOINIT: ready",
            "RUGO: halt ok",
        ],
    )
    assert second.count("DIAGSVC: snapshot") >= 2
    assert _has_task_line(second, "timesvc", ["dom=1", "cap=0", "fd=0", "sock=0"])
    assert _has_task_line(second, "diagsvc", ["dom=2", "cap=0", "fd=0", "sock=0"])
    assert _has_task_line(second, "shell", ["dom=3", "cap=3", "fd=0", "sock=0"])
