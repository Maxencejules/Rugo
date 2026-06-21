# Full-OS guide Part III acceptance: PC speaker audio.
#
# sys_ioctl op 3 programs PIT channel 2 to a tone and gates the speaker on,
# then reads port 0x61 back to confirm the enable bits took. beepprobe drives
# it at 440 Hz; the test asserts the divisor and the read-back gate bits.


def test_pc_speaker_beep(qemu_go_c4_runtime, find_in_order):
    boot, _disk_path = qemu_go_c4_runtime

    out = boot("probe beepprobe\nshutdown\n").stdout

    find_in_order(out, [
        # 440 Hz -> divisor 1193182/440 = 2711 = 0xA97; gate bits 0,1 set.
        "BEEP: freq=0x00000000000001B8 div=0x0000000000000A97 gate=0x0000000000000003",
        "BEEPPROBE: ok",
        "RUGO: halt ok",
    ])
    assert "BEEPPROBE: FAIL" not in out
