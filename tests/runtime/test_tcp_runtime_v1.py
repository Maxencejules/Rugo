# Phase 6 acceptance: real wire TCP from the default lane. The guest
# connects out through QEMU's user-mode network to a host-side listener
# owned by this test; the payload only round-trips if the kernel's TCP
# machine completes a real handshake and exchanges real segments.

import socket
import threading


def _echo_listener(server: socket.socket, result: dict) -> None:
    try:
        server.settimeout(20)
        conn, _addr = server.accept()
        conn.settimeout(10)
        data = b""
        while len(data) < len(b"rugo-tcp-hello"):
            chunk = conn.recv(64)
            if not chunk:
                break
            data += chunk
        result["got"] = data
        conn.sendall(b"tcp-hello-back")
        # Let the guest close first so its FIN path is exercised.
        try:
            conn.recv(64)
        except OSError:
            pass
        conn.close()
    except OSError as exc:
        result["error"] = str(exc)
    finally:
        server.close()


def test_wire_tcp_round_trip(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.bind(("127.0.0.1", 0))
    server.listen(1)
    port = server.getsockname()[1]

    result: dict = {}
    listener = threading.Thread(target=_echo_listener, args=(server, result))
    listener.start()

    out = boot(f"tcpcheck {port}\nshutdown\n").stdout
    listener.join(timeout=25)

    assert result.get("got") == b"rugo-tcp-hello", (
        f"host listener saw {result!r}\nserial:\n{out}"
    )
    # Anchors are single-write kernel/shell markers: the echoed command
    # line is typed char-by-char and may be spliced by async output.
    find_in_order(out, [
        "GOSH: session ready",
        "TCP: syn sent",
        "TCP: established",
        "NETT: tcp ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "NETT: tcp err" not in out
    assert "TCP: rst" not in out
