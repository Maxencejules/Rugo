# Phase 11 acceptance: DHCP and DNS clients (gap item 6 remainder).
# DHCP: the kernel broadcasts a real DISCOVER and parses the OFFER from
# QEMU's built-in DHCP server (no host network needed). DNS: the kernel
# sends a real A query through slirp to a resolver owned by this test,
# which answers 1.2.3.4 - the parsed address only appears if the whole
# query/response cycle crossed the wire.

import socket
import struct
import threading


def _find_in_order(serial: str, markers: list[str]) -> None:
    pos = -1
    for marker in markers:
        pos = serial.find(marker, pos + 1)
        assert pos != -1, f"Missing '{marker}' in serial output.\nFull output:\n{serial}"


class _DnsResponder:
    def __init__(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.bind(("127.0.0.1", 0))
        self.sock.settimeout(0.5)
        self.port = self.sock.getsockname()[1]
        self.queries = []
        self.alive = True
        self.thread = threading.Thread(target=self._serve, daemon=True)
        self.thread.start()

    def _serve(self):
        while self.alive:
            try:
                data, addr = self.sock.recvfrom(512)
            except socket.timeout:
                continue
            except OSError:
                break
            if len(data) < 12:
                continue
            self.queries.append(data)
            txid = data[0:2]
            # Echo the question, answer with a compression pointer to
            # offset 12 and A = 1.2.3.4.
            question_end = 12
            while question_end < len(data) and data[question_end] != 0:
                question_end += data[question_end] + 1
            question_end += 5
            question = data[12:question_end]
            resp = txid + b"\x81\x80" + struct.pack(">HHHH", 1, 1, 0, 0)
            resp += question
            resp += b"\xc0\x0c" + struct.pack(">HHIH", 1, 1, 60, 4)
            resp += bytes([1, 2, 3, 4])
            self.sock.sendto(resp, addr)

    def stop(self):
        self.alive = False
        self.thread.join(timeout=3)
        self.sock.close()


def test_dhcp_offer_and_dns_resolution(qemu_go_c4_runtime):
    boot, _disk_path = qemu_go_c4_runtime

    responder = _DnsResponder()
    try:
        out = boot(
            "dhcpcheck\n"
            f"dnscheck rugo.test {responder.port}\n"
            "shutdown\n"
        ).stdout
    finally:
        responder.stop()

    assert responder.queries, "the host-side DNS responder never saw a query"
    _find_in_order(out, [
        # slirp offers the guest its canonical address
        "DHCP: offer ip=0x000000000A00020F",
        "NETD: dhcp ok",
        # 1.2.3.4 from the test's resolver
        "DNS: a=0x0000000001020304",
        "NETD: dns ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "NETD: dhcp err" not in out
    assert "NETD: dns err" not in out
    assert "GOINIT: err" not in out
