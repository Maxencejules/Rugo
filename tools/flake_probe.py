"""Boot os-go.iso repeatedly and save any serial output that lacks a marker."""

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tests"))
import conftest  # noqa: E402

REQUIRED = [
    "SVC: timesvc stopped",
    "SVC: diagsvc stopped",
    "SVC: pkgsvc stopped",
    "GOSVCM: reap timesvc stopped res=ordered-stop",
    "GOSVCM: reap diagsvc stopped res=ordered-stop",
    "GOSVCM: reap pkgsvc stopped res=ordered-stop",
    "GOINIT: result shutdown-clean",
    "GOINIT: ready",
    "RUGO: halt ok",
]


def main() -> int:
    iso = str(Path(__file__).resolve().parents[1] / "out" / "os-go.iso")
    disk = str(Path(__file__).resolve().parents[1] / "out" / "flake-probe.img")
    if not Path(disk).is_file():
        Path(disk).write_bytes(b"\x00" * (1 << 20))
    bad = 0
    for i in range(14):
        result = conftest._boot_iso_with_disk_and_net(
            iso, disk, input_text="health\nshutdown\n"
        )
        missing = [m for m in REQUIRED if m not in result.stdout]
        if missing:
            bad += 1
            out = Path(__file__).resolve().parents[1] / "out" / f"flake-{i}.log"
            out.write_text(
                "MISSING: " + ", ".join(missing) + "\n---\n" + result.stdout,
                encoding="utf-8",
            )
            print(f"run {i}: MISSING {missing}")
        else:
            print(f"run {i}: ok")
    print(f"bad: {bad}/14")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
