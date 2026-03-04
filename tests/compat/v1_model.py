"""Deterministic reference model for M8 PR-2 compatibility semantics."""

from dataclasses import dataclass, field


ELF64_MAGIC = b"\x7fELF"
USER_LIMIT = 0x0000_8000_0000_0000


def _u16_le(buf, off):
    return int.from_bytes(buf[off : off + 2], "little")


def _u32_le(buf, off):
    return int.from_bytes(buf[off : off + 4], "little")


def _u64_le(buf, off):
    return int.from_bytes(buf[off : off + 8], "little")


def validate_elf64_image(image):
    if len(image) < 64:
        return False
    if image[:4] != ELF64_MAGIC:
        return False
    if image[4] != 2 or image[5] != 1 or image[6] != 1:
        return False

    entry = _u64_le(image, 24)
    phoff = _u64_le(image, 32)
    phentsize = _u16_le(image, 54)
    phnum = _u16_le(image, 56)

    if entry == 0 or entry >= USER_LIMIT:
        return False
    if phnum == 0 or phnum > 32:
        return False
    if phentsize < 56:
        return False
    if phoff + phentsize * phnum > len(image):
        return False

    load_count = 0
    for i in range(phnum):
        base = phoff + i * phentsize
        p_type = _u32_le(image, base + 0)
        if p_type != 1:
            continue
        load_count += 1
        p_offset = _u64_le(image, base + 8)
        p_vaddr = _u64_le(image, base + 16)
        p_filesz = _u64_le(image, base + 32)
        p_memsz = _u64_le(image, base + 40)
        p_align = _u64_le(image, base + 48)

        if p_memsz < p_filesz:
            return False
        if p_align != 0 and (p_align & (p_align - 1)) != 0:
            return False
        if p_offset + p_filesz > len(image):
            return False
        if p_vaddr >= USER_LIMIT:
            return False
        if p_vaddr + p_memsz > USER_LIMIT:
            return False

    return load_count > 0


@dataclass
class ProcessImage:
    argv: list
    envp: list
    exited: bool = False
    exit_status: int = 0
    pid: int = 1


class ProcessModel:
    def __init__(self):
        self.proc = ProcessImage(argv=[], envp=[])

    def execve(self, argv, envp):
        if not argv or argv[0] == "":
            return -1
        self.proc = ProcessImage(argv=list(argv), envp=list(envp))
        return 0

    def startup_contract(self):
        argv = self.proc.argv + [None]
        envp = self.proc.envp + [None]
        auxv = [
            ("AT_PHDR", 0x400040),
            ("AT_PHENT", 56),
            ("AT_PHNUM", 1),
            ("AT_PAGESZ", 4096),
            ("AT_ENTRY", 0x400000),
            ("AT_NULL", 0),
        ]
        return {"argv": argv, "envp": envp, "auxv": auxv}

    def exit(self, status):
        self.proc.exited = True
        self.proc.exit_status = status & 0xFF
        return 0

    def waitpid(self, pid, options=0):
        if options != 0:
            return -1, None
        if pid not in (-1, self.proc.pid):
            return -1, None
        if not self.proc.exited:
            return -1, None
        self.proc.exited = False
        return self.proc.pid, self.proc.exit_status


@dataclass
class FdEntry:
    kind: str
    offset: int = 0


class FdTableModel:
    def __init__(self):
        self.entries = {
            0: FdEntry("console"),
            1: FdEntry("console"),
            2: FdEntry("console"),
        }
        self.next_fd = 3
        self.compat_data = b"compat v1 hello\n"
        self.console_log = bytearray()

    def _alloc(self, kind):
        fd = self.next_fd
        while fd in self.entries:
            fd += 1
        if fd >= 16:
            return -1
        self.entries[fd] = FdEntry(kind=kind)
        self.next_fd = fd + 1
        return fd

    def open(self, path):
        if path == "/dev/console":
            return self._alloc("console")
        if path == "/compat/hello.txt":
            return self._alloc("compat_file")
        return -1

    def read(self, fd, length):
        ent = self.entries.get(fd)
        if ent is None or length < 0:
            return -1, b""
        if ent.kind != "compat_file":
            return -1, b""
        data = self.compat_data[ent.offset : ent.offset + length]
        ent.offset += len(data)
        return len(data), data

    def write(self, fd, data):
        ent = self.entries.get(fd)
        if ent is None:
            return -1
        if ent.kind != "console":
            return -1
        self.console_log.extend(data)
        return len(data)

    def close(self, fd):
        if fd < 3:
            return -1
        if fd not in self.entries:
            return -1
        del self.entries[fd]
        return 0

    def poll(self, pollfds):
        ready = 0
        out = []
        for fd, events in pollfds:
            ent = self.entries.get(fd)
            revents = 0
            if ent is None:
                revents = 0x0008  # POLLERR
            elif ent.kind == "console" and events & 0x0004:
                revents = 0x0004  # POLLOUT
            elif (
                ent.kind == "compat_file"
                and events & 0x0001
                and ent.offset < len(self.compat_data)
            ):
                revents = 0x0001  # POLLIN
            if revents:
                ready += 1
            out.append((fd, events, revents))
        return ready, out


CLOCK_REALTIME = 0
CLOCK_MONOTONIC = 1
SIGINT = 2
SIGTERM = 15


@dataclass
class SignalAction:
    handler: str = "SIG_DFL"
    restart: bool = False


class TimeSignalModel:
    def __init__(self):
        self.pid = 1
        self._mono_ns = 1_000_000_000
        self._real_ns = 1_700_000_000_000_000_000
        self._actions = {
            SIGINT: SignalAction(),
            SIGTERM: SignalAction(),
        }
        self.pending = []
        self.last_delivery = None

    def clock_gettime(self, clock_id):
        if clock_id == CLOCK_MONOTONIC:
            ns = self._mono_ns
        elif clock_id == CLOCK_REALTIME:
            ns = self._real_ns
        else:
            return -1, None
        return 0, (ns // 1_000_000_000, ns % 1_000_000_000)

    def sigaction(self, signum, handler, restart=False):
        old = self._actions.get(signum)
        if old is None:
            return -1, None
        self._actions[signum] = SignalAction(handler=handler, restart=restart)
        return 0, old

    def kill(self, pid, signum):
        action = self._actions.get(signum)
        if action is None:
            return -1
        if pid != self.pid:
            return -1
        if action.handler == "SIG_IGN":
            self.last_delivery = {
                "pid": pid,
                "signum": signum,
                "handler": "SIG_IGN",
                "restart": action.restart,
            }
            return 0

        self.pending.append(
            {
                "pid": pid,
                "signum": signum,
                "handler": action.handler,
                "restart": action.restart,
            }
        )
        return 0

    def deliver_next_signal(self):
        if not self.pending:
            return None
        evt = self.pending.pop(0)
        self.last_delivery = evt
        return evt

    def nanosleep(self, req_sec, req_nsec):
        if req_sec < 0 or req_nsec < 0 or req_nsec >= 1_000_000_000:
            return -1, (0, 0)

        req_ns = req_sec * 1_000_000_000 + req_nsec
        if self.pending:
            evt = self.pending.pop(0)
            self.last_delivery = evt
            if evt["restart"]:
                self._mono_ns += req_ns
                self._real_ns += req_ns
                return 0, (0, 0)
            return -1, (req_sec, req_nsec)

        self._mono_ns += req_ns
        self._real_ns += req_ns
        return 0, (0, 0)


@dataclass
class SocketState:
    family: str
    sock_type: str
    bound: tuple | None = None
    connected_to: tuple | None = None
    peer_fd: int | None = None
    listening: bool = False
    backlog: int = 0
    accept_queue: list = field(default_factory=list)
    recv_queue: list = field(default_factory=list)
    read_closed: bool = False
    write_closed: bool = False


class SocketModel:
    def __init__(self):
        self.entries = {}
        self.next_fd = 3
        self.next_ephemeral_port = 40_000

    def _alloc(self, entry):
        fd = self.next_fd
        while fd in self.entries:
            fd += 1
        if fd >= 64:
            return -1
        self.entries[fd] = entry
        self.next_fd = fd + 1
        return fd

    def _alloc_ephemeral(self):
        p = self.next_ephemeral_port
        self.next_ephemeral_port += 1
        return p

    def _find_bound(self, sock_type, addr):
        for fd, ent in self.entries.items():
            if ent.sock_type == sock_type and ent.bound == addr:
                return fd, ent
        return None, None

    def socket(self, family, sock_type):
        if family != "AF_INET":
            return -1
        if sock_type not in ("SOCK_STREAM", "SOCK_DGRAM"):
            return -1
        return self._alloc(SocketState(family=family, sock_type=sock_type))

    def bind(self, fd, addr):
        ent = self.entries.get(fd)
        if ent is None or ent.bound is not None:
            return -1
        _fd, bound_ent = self._find_bound(ent.sock_type, addr)
        if bound_ent is not None:
            return -1
        ent.bound = addr
        return 0

    def listen(self, fd, backlog):
        ent = self.entries.get(fd)
        if ent is None or ent.sock_type != "SOCK_STREAM" or ent.bound is None:
            return -1
        ent.listening = True
        ent.backlog = backlog if backlog > 0 else 1
        return 0

    def connect(self, fd, addr):
        ent = self.entries.get(fd)
        if ent is None:
            return -1

        if ent.sock_type == "SOCK_DGRAM":
            ent.connected_to = addr
            if ent.bound is None:
                ent.bound = ("127.0.0.1", self._alloc_ephemeral())
            return 0

        if ent.sock_type != "SOCK_STREAM":
            return -1

        listener_fd, listener = self._find_bound("SOCK_STREAM", addr)
        if listener is None or not listener.listening:
            return -1
        if len(listener.accept_queue) >= listener.backlog:
            return -1
        if ent.bound is None:
            ent.bound = ("127.0.0.1", self._alloc_ephemeral())

        server_fd = self._alloc(
            SocketState(
                family="AF_INET",
                sock_type="SOCK_STREAM",
                bound=addr,
                connected_to=ent.bound,
                peer_fd=fd,
            )
        )
        if server_fd < 0:
            return -1
        ent.connected_to = addr
        ent.peer_fd = server_fd
        listener.accept_queue.append(server_fd)
        return 0

    def accept(self, fd):
        listener = self.entries.get(fd)
        if listener is None or not listener.listening:
            return -1, None
        if not listener.accept_queue:
            return -1, None
        child_fd = listener.accept_queue.pop(0)
        child = self.entries.get(child_fd)
        peer_addr = child.connected_to if child is not None else None
        return child_fd, peer_addr

    def send(self, fd, data):
        ent = self.entries.get(fd)
        payload = bytes(data)
        if ent is None or ent.write_closed:
            return -1
        if ent.sock_type == "SOCK_DGRAM":
            if ent.connected_to is None:
                return -1
            return self.sendto(fd, payload, ent.connected_to)
        if ent.sock_type != "SOCK_STREAM" or ent.peer_fd is None:
            return -1
        peer = self.entries.get(ent.peer_fd)
        if peer is None or peer.read_closed:
            return -1
        peer.recv_queue.append((payload, ent.bound))
        return len(payload)

    def recv(self, fd, length):
        ent = self.entries.get(fd)
        if ent is None or ent.sock_type != "SOCK_STREAM" or ent.read_closed:
            return -1, b""
        if not ent.recv_queue or length < 0:
            return -1, b""
        payload, src = ent.recv_queue[0]
        chunk = payload[:length]
        if len(chunk) == len(payload):
            ent.recv_queue.pop(0)
        else:
            ent.recv_queue[0] = (payload[length:], src)
        return len(chunk), chunk

    def sendto(self, fd, data, addr):
        ent = self.entries.get(fd)
        payload = bytes(data)
        if ent is None or ent.sock_type != "SOCK_DGRAM" or ent.write_closed:
            return -1
        _target_fd, target = self._find_bound("SOCK_DGRAM", addr)
        if target is None:
            return -1
        if ent.bound is None:
            ent.bound = ("127.0.0.1", self._alloc_ephemeral())
        target.recv_queue.append((payload, ent.bound))
        return len(payload)

    def recvfrom(self, fd, length):
        ent = self.entries.get(fd)
        if ent is None or ent.sock_type != "SOCK_DGRAM" or ent.read_closed:
            return -1, b"", None
        if not ent.recv_queue or length < 0:
            return -1, b"", None
        payload, src = ent.recv_queue[0]
        chunk = payload[:length]
        if len(chunk) == len(payload):
            ent.recv_queue.pop(0)
        else:
            ent.recv_queue[0] = (payload[length:], src)
        return len(chunk), chunk, src

    def shutdown(self, fd, how):
        ent = self.entries.get(fd)
        if ent is None:
            return -1
        if how == 0:
            ent.read_closed = True
        elif how == 1:
            ent.write_closed = True
        elif how == 2:
            ent.read_closed = True
            ent.write_closed = True
        else:
            return -1
        return 0

    def poll(self, pollfds):
        ready = 0
        out = []
        for fd, events in pollfds:
            ent = self.entries.get(fd)
            revents = 0
            if ent is None:
                revents = 0x0008  # POLLERR
            else:
                if events & 0x0001:  # POLLIN
                    if ent.listening and ent.accept_queue:
                        revents |= 0x0001
                    elif ent.recv_queue and not ent.read_closed:
                        revents |= 0x0001
                if events & 0x0004 and not ent.write_closed:  # POLLOUT
                    revents |= 0x0004
            if revents:
                ready += 1
            out.append((fd, events, revents))
        return ready, out
