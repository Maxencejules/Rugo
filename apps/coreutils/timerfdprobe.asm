; timerfdprobe: timerfd proof (sys_time op 3 create + read/poll).
;
; Creates a 50 ms one-shot timerfd, confirms an immediate read returns 0
; (not yet armed-expired), sleeps 60 ms, then reads the 8-byte expiration
; count (must be 1). sys_time: rdi=op (2 nanosleep, 3 timerfd_create),
; rsi=arg. read=19 (rdi=fd, rsi=buf, rdx=len).

bits 64
default rel

section .text
global _start
_start:
    ; timerfd_create(50 ms)
    mov  edi, 3
    mov  rsi, 50000000
    mov  eax, 53
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax
    ; immediate read -> not expired -> 0
    mov  rdi, r12
    lea  rsi, [rel buf]
    mov  edx, 8
    mov  eax, 19
    int  0x80
    test rax, rax
    jnz  .fail
    ; sleep 60 ms
    mov  edi, 2
    mov  rsi, 60000000
    mov  eax, 53
    int  0x80
    ; read -> expired -> 8 bytes, count 1
    mov  rdi, r12
    lea  rsi, [rel buf]
    mov  edx, 8
    mov  eax, 19
    int  0x80
    cmp  rax, 8
    jne  .fail
    mov  rax, [rel buf]
    cmp  rax, 1
    jne  .fail
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
okmsg:   db "TIMERFDPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "TIMERFDPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
buf: resb 8
