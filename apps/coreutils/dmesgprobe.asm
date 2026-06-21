; dmesgprobe: kernel log (dmesg) ring-buffer proof.
;
; Full-OS guide Part V.11 (observability) / IV.10 (audit): the kernel mirrors
; every serial_write line into a ring buffer (sys_sysinfo op 4 reads it). The
; probe writes a unique cookie to the console (which the ring captures), reads
; the dmesg tail back, and echoes it. The test then sees the cookie TWICE,
; proving the kernel captured and returned its own log output.
;
; Syscall ABI: eax=0 write(rdi=ptr, rsi=len); eax=2 exit;
; eax=61 sysinfo(rdi=op, rsi=ptr, rdx=len) -> rax.

bits 64
default rel

section .text
global _start
_start:
    ; 1) emit the cookie (captured by the dmesg ring as it is written)
    lea  rdi, [rel cookie]
    mov  esi, cookie_len
    xor  eax, eax
    int  0x80

    ; 2) read the dmesg tail into buf. The console-write syscall caps at 256
    ;    bytes, so read a 200-byte tail that fits in one echo; the cookie just
    ;    written is the most recent ring entry, so it is in this tail.
    mov  edi, 4                 ; op 4 = dmesg read
    lea  rsi, [rel buf]
    mov  edx, 200
    mov  eax, 61
    int  0x80
    test rax, rax
    js   .fail                  ; u64::MAX -> error
    jz   .fail                  ; 0 bytes -> nothing captured

    ; 3) echo the captured dmesg tail back to the console
    lea  rdi, [rel buf]
    mov  esi, eax               ; len = bytes copied
    xor  eax, eax
    int  0x80

    ; 4) verdict
    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.fail:
    lea  rdi, [rel failmsg]
    mov  esi, failmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
cookie:  db "DMESGCOOKIE-7142", 10
cookie_len equ $ - cookie
okmsg:   db "DMESGPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "DMESGPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
buf:     resb 2048
