"""Compatibility Profile v1: time/signal executable checks."""

from pathlib import Path

import sys

sys.path.append(str(Path(__file__).resolve().parent))

from v1_model import CLOCK_MONOTONIC, SIGINT, SIGTERM, TimeSignalModel


def _profile_text():
    return (
        Path(__file__).resolve().parents[2] / "docs" / "abi" / "compat_profile_v1.md"
    ).read_text(encoding="utf-8")


def test_profile_declares_time_signal_subset():
    text = _profile_text()
    assert "### Time and signal subset (`required`)" in text


def test_clock_gettime_and_nanosleep_contract():
    model = TimeSignalModel()
    rc0, t0 = model.clock_gettime(CLOCK_MONOTONIC)
    assert rc0 == 0
    assert t0 is not None

    rc_sleep, rem = model.nanosleep(0, 50_000_000)
    assert rc_sleep == 0
    assert rem == (0, 0)

    rc1, t1 = model.clock_gettime(CLOCK_MONOTONIC)
    assert rc1 == 0
    assert t1 is not None
    assert t1[0] * 1_000_000_000 + t1[1] > t0[0] * 1_000_000_000 + t0[1]


def test_sigaction_and_kill_delivery_contract():
    model = TimeSignalModel()
    rc, old = model.sigaction(SIGTERM, "term_handler", restart=False)
    assert rc == 0
    assert old.handler == "SIG_DFL"

    assert model.kill(1, SIGTERM) == 0
    evt = model.deliver_next_signal()
    assert evt is not None
    assert evt["signum"] == SIGTERM
    assert evt["handler"] == "term_handler"
    assert evt["restart"] is False


def test_sleep_interrupt_and_restart_behavior():
    model = TimeSignalModel()

    assert model.sigaction(SIGTERM, "term_handler", restart=False)[0] == 0
    assert model.kill(1, SIGTERM) == 0
    assert model.nanosleep(0, 5_000_000) == (-1, (0, 5_000_000))

    assert model.sigaction(SIGINT, "int_handler", restart=True)[0] == 0
    assert model.kill(1, SIGINT) == 0
    assert model.nanosleep(0, 5_000_000) == (0, (0, 0))
