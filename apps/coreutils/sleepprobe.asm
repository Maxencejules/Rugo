; sleepprobe: nanosleep proof (sys_time op 2).
;
; Reads CLOCK_MONOTONIC, sleeps ~100 ms, reads it again, and verifies at
; least ~90 ms of monotonic time elapsed (the task was genuinely blocked
; and woken by the PIT, not spinning). sys_time: rdi=op (1 gettime,
; 2 nanosleep), rsi=arg.

bits 64
default rel

section .text
global _start
_start:
    ; t1 = monotonic ns
    mov  edi, 1
    xor  esi, esi
    mov  eax, 53
    int  0x80
    mov  r14, rax
    ; nanosleep(100 ms)
    mov  edi, 2
    mov  rsi, 100000000
    mov  eax, 53
    int  0x80
    ; t2 = monotonic ns
    mov  edi, 1
    xor  esi, esi
    mov  eax, 53
    int  0x80
    sub  rax, r14
    cmp  rax, 90000000
    jb   .fail
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
okmsg:   db "SLEEPPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "SLEEPPROBE: FAIL", 10
failmsg_len equ $ - failmsg
