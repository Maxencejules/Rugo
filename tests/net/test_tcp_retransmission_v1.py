"""M12 acceptance: TCP retransmission timer policy baseline."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from net_v1_model import RetransmissionPolicyModel  # noqa: E402


def test_retransmission_backoff_and_ack_reset():
    model = RetransmissionPolicyModel()
    assert model.send(256) == 256
    assert model.current_rto_ms == 200

    ev1 = model.tick(200)
    assert ev1 == ["retransmit"]
    assert model.retry_count == 1
    assert model.current_rto_ms == 400

    ev2 = model.tick(400)
    assert ev2 == ["retransmit"]
    assert model.retry_count == 2
    assert model.current_rto_ms == 800

    assert model.ack_all() == 0
    assert model.in_flight_bytes == 0
    assert model.retry_count == 0
    assert model.current_rto_ms == 200


def test_retransmission_retry_exhaustion_times_out():
    model = RetransmissionPolicyModel(max_retries=3)
    assert model.send(64) == 64

    assert model.tick(200) == ["retransmit"]
    assert model.tick(400) == ["retransmit"]
    final = model.tick(800)
    assert final == ["retransmit", "timeout"]
    assert model.failed is True


def test_no_timer_events_without_inflight_data():
    model = RetransmissionPolicyModel()
    assert model.tick(500) == []
