#!/usr/bin/env python3
"""Compile/run the actual no_std static ELF parser on the host, offline.

No Cargo dependency resolution or bare-metal target is needed. RUSTC (or
--rustc) may select an installed compiler; RUSTC_LINKER may select a host linker.
Outputs, compiler provenance and both test transcripts stay under out/.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(command: list[str], transcript: Path) -> None:
    result = subprocess.run(
        command, cwd=ROOT, text=True, capture_output=True, timeout=120
    )
    output = "$ " + repr(command) + "\n" + result.stdout + result.stderr
    transcript.write_text(output, encoding="utf-8")
    print(output, end="")
    if result.returncode:
        raise SystemExit(result.returncode)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", default=os.environ.get("RUSTC", "rustc"))
    parser.add_argument("--out", type=Path, default=ROOT / "out" / "exec-static-v1")
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    run([args.rustc, "--version", "--verbose"], out / "compiler.txt")
    flags = ["--edition=2021", "-D", "warnings"]
    if linker := os.environ.get("RUSTC_LINKER"):
        flags += ["-C", f"linker={linker}"]
    module = (ROOT / "kernel_rs" / "src" / "elf_static.rs").as_posix()
    no_std = out / "no_std.rs"
    no_std.write_text(
        f'#![no_std]\n#[path = "{module}"]\npub mod elf_static;\n', encoding="utf-8"
    )
    run(
        [
            args.rustc,
            *flags,
            "--crate-type",
            "lib",
            str(no_std),
            "-o",
            str(out / "no_std.rlib"),
        ],
        out / "no-std.txt",
    )
    for profile, optimization in [("debug", "0"), ("release", "3")]:
        binary = out / (profile + (".exe" if os.name == "nt" else ""))
        run(
            [
                args.rustc,
                *flags,
                "--test",
                "-C",
                f"opt-level={optimization}",
                str(ROOT / "tests" / "host" / "exec_static_v1.rs"),
                "-o",
                str(binary),
            ],
            out / f"{profile}-build.txt",
        )
        run([str(binary), "--test-threads=1"], out / f"{profile}-tests.txt")


if __name__ == "__main__":
    main()
