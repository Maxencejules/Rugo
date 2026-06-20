# Full-OS guide Part II.6 acceptance: TCP MSS option on the SYN.
#
# A real TCP stack advertises its Maximum Segment Size on the SYN so the peer never
# sends an oversized segment. The kernel's outbound segment builder now puts a 4-byte
# MSS option (kind 2, MSS 1460) on every SYN, raising the data offset to 6 (a 24-byte
# TCP header). At boot a self-test builds a SYN and confirms (a) the data offset is 6,
# (b) the MSS option bytes are present, and (c) the TCP checksum -- recomputed over the
# pseudo-header including the stored checksum field -- folds to 0, i.e. the segment is
# well-formed. The live TCP client uses the same builder, so the existing round-trip
# tests (test_tcp_runtime_v1 et al.) double as proof a slirp peer accepts the
# MSS-bearing SYN. "TCP: mss ok".


def test_tcp_syn_mss(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("shutdown\n").stdout

    find_in_order(out, [
        "TCP: mss ok",
        "GOINIT: result shutdown-clean",
        "RUGO: halt ok",
    ])
    assert "TCP: mss fail" not in out
