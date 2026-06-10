"""M12 acceptance: TCP state-machine baseline semantics."""

from pathlib import Path
import sys

sys.path.append(str(Path(__file__).resolve().parent))
from net_v1_model import TcpConnectionModel  # noqa: E402


def test_active_open_three_way_handshake_and_data_ack():
    model = TcpConnectionModel()
    assert model.state == "CLOSED"
    assert model.active_open() == 0
    assert model.state == "SYN_SENT"
    assert model.recv_syn_ack() == 0
    assert model.state == "ESTABLISHED"
    assert model.send_data(128) == 128
    assert model.unacked_bytes == 128
    assert model.recv_ack() == 0
    assert model.unacked_bytes == 0


def test_passive_open_and_graceful_close_path():
    model = TcpConnectionModel()
    assert model.listen() == 0
    assert model.recv_syn() == 0
    assert model.state == "SYN_RECEIVED"
    assert model.recv_ack() == 0
    assert model.state == "ESTABLISHED"
    assert model.close() == 0
    assert model.state == "FIN_WAIT_1"
    assert model.recv_ack() == 0
    assert model.state == "FIN_WAIT_2"
    assert model.recv_fin() == 0
    assert model.state == "TIME_WAIT"
    assert model.time_wait_expire() == 0
    assert model.state == "CLOSED"


def test_reset_forces_deterministic_closed_state():
    model = TcpConnectionModel()
    assert model.active_open() == 0
    assert model.recv_syn_ack() == 0
    assert model.state == "ESTABLISHED"
    assert model.recv_rst() == 0
    assert model.state == "CLOSED"
    assert model.unacked_bytes == 0
