#!/usr/bin/env python3
"""Emit M11 runtime/toolchain contract and reproducibility artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Tuple


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def _find_tool(*candidates: str) -> str | None:
    for cand in candidates:
        resolved = shutil.which(cand)
        if resolved:
            return resolved
        p = Path(cand)
        if p.is_file():
            return str(p)
    return None


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _parse_env_file(path: Path) -> Dict[str, str]:
    data: Dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        data[key.strip()] = value.strip()
    return data


def _extract_abi_window(policy_doc: Path) -> str:
    text = policy_doc.read_text(encoding="utf-8")
    match = re.search(r"Stability window:\s*`([^`]+)`", text)
    if match:
        return match.group(1)
    raise RuntimeError(
        f"Could not extract stability window from {policy_doc}"
    )


def _run_stock_go_builder(root: Path) -> None:
    script_error: str | None = None
    bash_bin = _find_tool(
        "bash",
        "C:/Program Files/Git/bin/bash.exe",
        "C:/Program Files/Git/usr/bin/bash.exe",
        "C:/Progra~1/Git/bin/bash.exe",
        "C:/Progra~1/Git/usr/bin/bash.exe",
        "/usr/bin/bash",
        "/bin/bash",
        "C:/WINDOWS/system32/bash.exe",
        "/mnt/c/WINDOWS/system32/bash.exe",
    )
    if bash_bin:
        cmd = [bash_bin, "tools/build_go_std_spike.sh"]
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
        )
        if proc.returncode == 0:
            return
        script_error = (
            "build_go_std_spike.sh failed\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )

    # Fallback for environments where shell-script line endings or bash setup
    # prevent script execution: run the stock-Go builder directly.
    go_bin = _find_tool(
        "go",
        "go.exe",
        "/mnt/c/Progra~1/Go/bin/go.exe",
        "/mnt/c/Program Files/Go/bin/go.exe",
        "C:/Program Files/Go/bin/go.exe",
    )
    if not go_bin:
        if script_error:
            raise RuntimeError(script_error)
        raise RuntimeError("go tool not found; cannot run stock-Go builder")

    env = os.environ.copy()
    env["GOENV"] = "off"
    env["CGO_ENABLED"] = "0"
    cache_dir = root / "out" / "go-build-cache"
    tmp_dir = root / "out" / "go-tmp"
    cache_dir.mkdir(parents=True, exist_ok=True)
    tmp_dir.mkdir(parents=True, exist_ok=True)
    env["GOCACHE"] = str(cache_dir)
    env["GOTMPDIR"] = str(tmp_dir)
    env.pop("CC", None)
    env.pop("CXX", None)
    proc = subprocess.run(
        [go_bin, "run", "./tools/gostd_stock_builder/main.go"],
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        details = (
            "direct go builder failed\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )
        if script_error:
            details = f"{script_error}\n{details}"
        raise RuntimeError(details)


def _ensure_stock_go_artifacts(root: Path, refresh: bool) -> Tuple[Path, Path]:
    out_dir = root / "out"
    out_dir.mkdir(parents=True, exist_ok=True)
    bin_path = out_dir / "gostd.bin"
    contract_path = out_dir / "gostd-contract.env"

    if refresh or not bin_path.is_file() or not contract_path.is_file():
        _run_stock_go_builder(root)

    if not bin_path.is_file():
        raise RuntimeError(f"Missing artifact: {bin_path}")
    if not contract_path.is_file():
        raise RuntimeError(f"Missing artifact: {contract_path}")
    return bin_path, contract_path


def _contract_report(root: Path, refresh: bool) -> Dict[str, str]:
    bin_path, stock_contract_path = _ensure_stock_go_artifacts(root, refresh=refresh)
    stock_contract = _parse_env_file(stock_contract_path)
    policy_doc = root / "docs" / "runtime" / "abi_stability_policy_v1.md"

    required_keys = [
        "GOOS",
        "GOARCH",
        "STOCK_GO_VERSION",
        "STOCK_GO_HOST_GOOS",
        "STOCK_GO_HOST_GOARCH",
    ]
    missing = [k for k in required_keys if k not in stock_contract]
    if missing:
        raise RuntimeError(
            f"Missing required key(s) in {stock_contract_path}: {', '.join(missing)}"
        )

    return {
        "RUNTIME_TOOLCHAIN_SCHEMA": "rugo.runtime_toolchain_contract.v1",
        "RUNTIME_TOOLCHAIN_VERSION": "v1",
        "RUNTIME_PORT_CONTRACT": "docs/runtime/port_contract_v1.md",
        "RUNTIME_ABI_POLICY": "docs/runtime/abi_stability_policy_v1.md",
        "RUNTIME_ABI_WINDOW": _extract_abi_window(policy_doc),
        "GOOS": stock_contract["GOOS"],
        "GOARCH": stock_contract["GOARCH"],
        "STOCK_GO_VERSION": stock_contract["STOCK_GO_VERSION"],
        "STOCK_GO_HOST_GOOS": stock_contract["STOCK_GO_HOST_GOOS"],
        "STOCK_GO_HOST_GOARCH": stock_contract["STOCK_GO_HOST_GOARCH"],
        "GOSTD_BIN_SHA256": _sha256(bin_path),
        "GOSTD_CONTRACT_SHA256": _sha256(stock_contract_path),
        "HOST_PLATFORM": platform.platform(),
        "HOST_MACHINE": platform.machine(),
        "HOST_PYTHON": platform.python_version(),
    }


def _write_env(path: Path, data: Dict[str, str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [f"{k}={v}" for k, v in sorted(data.items())]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _repro_report(root: Path) -> Dict[str, object]:
    first = _contract_report(root, refresh=True)
    second = _contract_report(root, refresh=True)
    equal = (
        first["GOSTD_BIN_SHA256"] == second["GOSTD_BIN_SHA256"]
        and first["GOSTD_CONTRACT_SHA256"] == second["GOSTD_CONTRACT_SHA256"]
    )
    return {
        "schema": "rugo.runtime_toolchain_repro.v1",
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "hashes_equal": equal,
        "first": {
            "gostd_bin_sha256": first["GOSTD_BIN_SHA256"],
            "gostd_contract_sha256": first["GOSTD_CONTRACT_SHA256"],
        },
        "second": {
            "gostd_bin_sha256": second["GOSTD_BIN_SHA256"],
            "gostd_contract_sha256": second["GOSTD_CONTRACT_SHA256"],
        },
        "goos": first["GOOS"],
        "goarch": first["GOARCH"],
        "stock_go_version": first["STOCK_GO_VERSION"],
    }


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--out",
        default="out/runtime-toolchain-contract.env",
        help="output file path",
    )
    p.add_argument(
        "--repro",
        action="store_true",
        help="emit reproducibility JSON report instead of env contract",
    )
    p.add_argument(
        "--refresh",
        action="store_true",
        help="force rebuilding stock-Go artifact before contract emission",
    )
    return p


def main(argv: list[str] | None = None) -> int:
    args = _build_parser().parse_args(argv)
    root = _repo_root()
    out_path = root / args.out if not os.path.isabs(args.out) else Path(args.out)

    if args.repro:
        report = _repro_report(root)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"runtime-toolchain-repro: {out_path}")
        print(f"hashes_equal: {report['hashes_equal']}")
        return 0 if report["hashes_equal"] else 1

    report = _contract_report(root, refresh=args.refresh)
    _write_env(out_path, report)
    print(f"runtime-toolchain-contract: {out_path}")
    print(f"go: {report['STOCK_GO_VERSION']}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(1)
