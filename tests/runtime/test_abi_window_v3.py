"""M21 PR-1: deprecation window policy enforcement checks."""

from __future__ import annotations

import re
from pathlib import Path
from typing import Dict, Iterable, Tuple


ROOT = Path(__file__).resolve().parents[2]

WINDOW_RE = re.compile(r"Minimum tagged-release window:\s*(\d+)")
DEPRECATION_ROW_RE = re.compile(
    r"^\|\s*`([^`]+)`\s*\|\s*(active|deprecated|removed)\s*\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|",
    re.IGNORECASE,
)
RELEASE_RE = re.compile(r"^v(\d+)\.(\d+)$")


def _read(relpath: str) -> str:
    return (ROOT / relpath).read_text(encoding="utf-8")


def _parse_release(tag: str) -> Tuple[int, int] | None:
    match = RELEASE_RE.match(tag.strip())
    if not match:
        return None
    return int(match.group(1)), int(match.group(2))


def _release_gap(old_release: str, new_release: str) -> int | None:
    old_parsed = _parse_release(old_release)
    new_parsed = _parse_release(new_release)
    if old_parsed is None or new_parsed is None:
        return None
    old_major, old_minor = old_parsed
    new_major, new_minor = new_parsed
    if new_major < old_major:
        return -1
    if new_major > old_major:
        return 10_000 + (new_major - old_major) * 100 + new_minor
    return new_minor - old_minor


def _iter_deprecation_rows(text: str) -> Iterable[Dict[str, str]]:
    for line in text.splitlines():
        match = DEPRECATION_ROW_RE.match(line.strip())
        if not match:
            continue
        yield {
            "symbol": match.group(1).strip(),
            "state": match.group(2).strip().lower(),
            "deprecated_in": match.group(3).strip(),
            "earliest_removal": match.group(4).strip(),
            "replacement": match.group(5).strip(),
        }


def test_deprecation_window_policy_is_explicit():
    policy = _read("docs/runtime/deprecation_window_policy_v1.md")
    assert "Minimum tagged-release window: 3." in policy
    assert "Minimum calendar window: 180 days." in policy
    assert "Earliest removal release must be at least" in policy


def test_syscall_v3_deprecation_entries_satisfy_policy():
    policy = _read("docs/runtime/deprecation_window_policy_v1.md")
    syscall_doc = _read("docs/abi/syscall_v3.md")

    window_match = WINDOW_RE.search(policy)
    assert window_match is not None, "missing tagged-release window token"
    min_window = int(window_match.group(1))

    assert "Deprecation registry (v3 line)" in syscall_doc
    assert "No syscalls are deprecated in v3.0." in syscall_doc

    for row in _iter_deprecation_rows(syscall_doc):
        if row["state"] not in {"deprecated", "removed"}:
            continue
        assert row["replacement"].lower() not in {"", "n/a", "none", "-"}
        gap = _release_gap(row["deprecated_in"], row["earliest_removal"])
        assert gap is not None, f"invalid release tags in row: {row}"
        assert gap >= min_window, f"deprecation window too short for {row['symbol']}"

