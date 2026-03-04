"""Compatibility Profile v1: socket subset executable checks."""

from pathlib import Path

import sys

sys.path.append(str(Path(__file__).resolve().parent))

from v1_model import SocketModel


def _profile_text():
    return (
        Path(__file__).resolve().parents[2] / "docs" / "abi" / "compat_profile_v1.md"
    ).read_text(encoding="utf-8")


def test_profile_declares_socket_subset():
    text = _profile_text()
    assert "### Socket API subset (`required`)" in text


def test_stream_socket_lifecycle_contract():
    model = SocketModel()
    srv = model.socket("AF_INET", "SOCK_STREAM")
    cli = model.socket("AF_INET", "SOCK_STREAM")
    assert srv >= 3
    assert cli >= 3

    bind_addr = ("10.0.2.15", 7001)
    assert model.bind(srv, bind_addr) == 0
    assert model.listen(srv, backlog=4) == 0

    assert model.connect(cli, bind_addr) == 0
    ready0, entries0 = model.poll([(srv, 0x0001)])
    assert ready0 == 1
    assert entries0[0][2] == 0x0001

    acc_fd, peer = model.accept(srv)
    assert acc_fd >= 3
    assert peer is not None

    assert model.send(cli, b"ping") == 4
    ready1, entries1 = model.poll([(acc_fd, 0x0001), (cli, 0x0004)])
    assert ready1 == 2
    assert entries1[0][2] == 0x0001
    assert entries1[1][2] == 0x0004

    n, data = model.recv(acc_fd, 16)
    assert n == 4
    assert data == b"ping"

    assert model.shutdown(cli, 1) == 0
    ready2, entries2 = model.poll([(cli, 0x0004)])
    assert ready2 == 0
    assert entries2[0][2] == 0


def test_datagram_sendto_recvfrom_and_poll_contract():
    model = SocketModel()
    srv = model.socket("AF_INET", "SOCK_DGRAM")
    cli = model.socket("AF_INET", "SOCK_DGRAM")
    assert srv >= 3
    assert cli >= 3

    srv_addr = ("10.0.2.15", 7002)
    cli_addr = ("10.0.2.15", 7003)
    assert model.bind(srv, srv_addr) == 0
    assert model.bind(cli, cli_addr) == 0

    assert model.sendto(cli, b"udp", srv_addr) == 3
    ready, entries = model.poll([(srv, 0x0001), (99, 0x0001)])
    assert ready == 2
    assert entries[0][2] == 0x0001
    assert entries[1][2] == 0x0008

    n, data, src = model.recvfrom(srv, 64)
    assert n == 3
    assert data == b"udp"
    assert src == cli_addr
