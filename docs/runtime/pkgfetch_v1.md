# Package fetch over TCP — contract v1

Status: boot-verified via `make test-pkgfetch-v1` (go lane, virtio-net + slirp)
Source: `kernel_rs/src/net.rs` (`pkg_fetch_arm`, `pkg_fetch_tick`,
`pkg_fetch_start`, `pkg_fetch_poll`); boot arming in `kernel_rs/src/lib.rs`
(reads the request sector); PIT driver in `r4_timer_preempt`.
Proof: `tests/runtime/test_pkgfetch_v1.py`.

Full-OS guide Part V.11 (package manager): the **network-download core** of a
package manager — connect out to a repo host over TCP, download a framed package,
and **content-verify** it (magic + checksum). It reuses the wire TCP client
([`tcp_rto_v1.md`](tcp_rto_v1.md)) and the DHCP-style start/poll pattern.

## Behaviour

A fetch is **armed by a request record on disk** (so ordinary boots never attempt
one): at the end of the go-lane boot self-tests the kernel reads **sector 16**;
if it begins with `"PKGREQ"` followed by a little-endian `u16` port, it calls
`pkg_fetch_arm(port)` and prints `PKG: fetch armed`.

The PIT-tick handler (which already pumps the wire RX + RTO) then drives the
fetch (`pkg_fetch_tick`):

1. once the NIC is up, `pkg_fetch_start` connects the wire TCP to the slirp
   gateway **10.0.2.2:port** (slirp forwards `guest→10.0.2.2:port` to the host's
   `127.0.0.1:port`, the same path the `tcpcheck` client uses);
2. the handshake advances over subsequent ticks (ARP → SYN → established) — the
   wire TCP's own RTO retransmits a lost SYN; a long give-up bound (`PKG: fetch
   timeout FAIL`) bounds a dead server;
3. `pkg_fetch_poll` drains the receive buffer into a package buffer and, once the
   whole framed package has arrived, verifies it.

The package wire format is `"RUGOPKG1"` (8) | `u32` payload length | payload |
`u32` checksum (sum of payload bytes, wrapping). On success the kernel prints
`PKG: fetched len=0x<n> ok`; on a bad checksum / early close / oversize / timeout
it prints a `PKG: fetch … FAIL` marker. The connection is then closed.

## Acceptance

`make test-pkgfetch-v1`: the test starts a host repo server (a framed 900-byte
package), writes the `PKGREQ` record to sector 16 of the boot disk, boots the go
lane (virtio-net + slirp), and holds the session open until a `PKG:` completion
marker appears. The transcript shows `PKG: fetch armed`, `TCP: syn sent`, and
`PKG: fetched len=0x0000000000000384 ok` (900 payload bytes, checksum matched),
reaching `GOINIT: result shutdown-clean` / `RUGO: halt ok` — with no
`PKG: fetch … FAIL`, and the host confirms it served the 916-byte package.

## v1 boundary / carry-forward

- **Fetch + content-verify a single package over TCP** (a real multi-segment
  download: 916 B > the 512 B MSS). What remains for a full package manager: a
  request/selection protocol (the server here pushes one package on connect),
  multiple packages + a repo index, signature verification (vs a plain checksum),
  unpacking + install into the file tree, and dependency resolution /
  self-hosting.
- The fetch is armed by an on-disk request record (a test/automation hook); a
  user-facing `pkg` command is carry-forward (the Go shell image is at its size
  cap, and spawned apps are storage-only — a network-capable package-manager app
  needs a capability-manifest mechanism).
