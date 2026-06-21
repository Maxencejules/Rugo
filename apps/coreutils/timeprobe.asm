; timeprobe: clock proof (sys_time, id 53).
;
; Reads CLOCK_MONOTONIC twice across a busy interval and proves it strictly
; advanced (the PIT preempts the busy loop, ticking the clock), then reads
; CLOCK_REALTIME and proves it is a plausible recent Unix timestamp.
;
; sys_time: rdi=op, rsi=a2. op 1 = clock_gettime(clockid) ->
;   clockid 0 = MONOTONIC nanoseconds, clockid 1 = REALTIME unix seconds.

bits 64
default rel

section .text
global _start
_start:
    ; t1 = monotonic
    mov  edi, 1
    xor  esi, esi
    mov  eax, 53
    int  0x80
    mov  r14, rax
    ; busy interval - PIT ticks advance the monotonic clock meanwhile
    mov  rcx, 60000000
.spin:
    dec  rcx
    jnz  .spin
    ; t2 = monotonic
    mov  edi, 1
    xor  esi, esi
    mov  eax, 53
    int  0x80
    cmp  rax, r14
    jbe  .mfail
    lea  rdi, [rel mok]
    mov  esi, mok_len
    xor  eax, eax
    int  0x80
    ; realtime
    mov  edi, 1
    mov  esi, 1
    mov  eax, 53
    int  0x80
    cmp  rax, 1700000000        ; after 2023-11 -> RTC is sane
    jbe  .rfail
    lea  rdi, [rel rok]
    mov  esi, rok_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

.mfail:
    lea  rdi, [rel mfail]
    mov  esi, mfail_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.rfail:
    lea  rdi, [rel rfail]
    mov  esi, rfail_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
mok:   db "TIMEPROBE: monotonic ok", 10
mok_len equ $ - mok
rok:   db "TIMEPROBE: realtime ok", 10
rok_len equ $ - rok
mfail: db "TIMEPROBE: monotonic FAIL", 10
mfail_len equ $ - mfail
rfail: db "TIMEPROBE: realtime FAIL", 10
rfail_len equ $ - rfail
