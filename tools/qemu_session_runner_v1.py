#!/usr/bin/env python3
"""Run QEMU with a TCP-backed serial console and inject input after a marker."""

from __future__ import annotations

import argparse
import queue
import socket
import subprocess
import sys
import threading
import time
from pathlib import Path


def _pump_lines(stream, out_queue: "queue.Queue[str | None]") -> None:
    try:
        for line in iter(stream.readline, ""):
            out_queue.put(line)
    finally:
        stream.close()
        out_queue.put(None)


def _pick_serial_port() -> int:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])
    finally:
        sock.close()


def _replace_serial_stdio(command: list[str], port: int) -> list[str]:
    updated: list[str] = []
    replaced = False
    index = 0
    while index < len(command):
        if command[index] == "-serial" and index + 1 < len(command):
            updated.extend(
                [
                    "-serial",
                    f"tcp:127.0.0.1:{port},server=on,wait=off",
                ]
            )
            index += 2
            replaced = True
            continue
        updated.append(command[index])
        index += 1
    if not replaced:
        updated.extend(["-serial", f"tcp:127.0.0.1:{port},server=on,wait=off"])
    return updated


def _connect_serial(
    port: int,
    process: subprocess.Popen[str],
    timeout_seconds: float,
) -> socket.socket:
    start = time.monotonic()
    while True:
        if process.poll() is not None:
            raise RuntimeError("QEMU exited before the serial console became ready")
        try:
            conn = socket.create_connection(("127.0.0.1", port), timeout=0.5)
            conn.settimeout(0.1)
            return conn
        except OSError:
            if timeout_seconds > 0 and (time.monotonic() - start) > timeout_seconds:
                raise RuntimeError("Timed out waiting for the QEMU serial console")
            time.sleep(0.1)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stdin-file", required=True)
    parser.add_argument("--marker", default="GOSH: session ready")
    parser.add_argument("--timeout", type=float, default=0.0)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()

    if not args.command:
        parser.error("missing command to run")

    command = args.command
    if command[0] == "--":
        command = command[1:]
    if not command:
        parser.error("missing command to run")

    input_text = Path(args.stdin_file).read_text(encoding="utf-8")
    serial_port = _pick_serial_port()
    command = _replace_serial_stdio(command, serial_port)
    process = subprocess.Popen(
        command,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    assert process.stdout is not None

    out_queue: "queue.Queue[str | None]" = queue.Queue()
    reader = threading.Thread(
        target=_pump_lines,
        args=(process.stdout, out_queue),
        daemon=True,
    )
    reader.start()

    try:
        serial = _connect_serial(serial_port, process, args.timeout or 10.0)
    except RuntimeError as exc:
        process.kill()
        reader.join(timeout=1.0)
        sys.stdout.write(f"{exc}\n")
        sys.stdout.flush()
        return process.returncode if process.returncode is not None else 1

    start = time.monotonic()
    input_sent = False
    output_closed = False
    serial_closed = False
    serial_buffer = ""

    while True:
        try:
            line = out_queue.get(timeout=0.1)
        except queue.Empty:
            line = None

        if line is None:
            if not output_closed and reader.is_alive():
                line = None
            else:
                output_closed = True
        else:
            sys.stdout.write(line)
            sys.stdout.flush()

        if not serial_closed:
            try:
                chunk = serial.recv(4096)
            except socket.timeout:
                chunk = b""
            except OSError:
                serial_closed = True
                chunk = b""
            if chunk:
                serial_buffer += chunk.decode("utf-8", errors="replace")
                while "\n" in serial_buffer:
                    raw_line, serial_buffer = serial_buffer.split("\n", 1)
                    clean_line = raw_line.rstrip("\r")
                    sys.stdout.write(clean_line + "\n")
                    sys.stdout.flush()
                    if not input_sent and args.marker in clean_line:
                        serial.sendall(input_text.encode("utf-8"))
                        input_sent = True
            elif chunk == b"" and not process.poll() is None:
                serial_closed = True

        if args.timeout > 0 and (time.monotonic() - start) > args.timeout:
            process.kill()
            serial.close()
            reader.join(timeout=1.0)
            return 124

        if process.poll() is not None and output_closed:
            if serial_buffer:
                sys.stdout.write(serial_buffer.rstrip("\r") + "\n")
                sys.stdout.flush()
                serial_buffer = ""
            break

    serial.close()
    reader.join(timeout=1.0)
    return process.returncode if process.returncode is not None else 1


if __name__ == "__main__":
    raise SystemExit(main())
