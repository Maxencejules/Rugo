; sysinfoprobe: /proc-style metrics proof (sys_sysinfo, id 61).
;
; op 1 = live task count (>=1, at least this probe), op 2 = free frames
; (>0), op 3 = uptime ticks (advances). rdi=op -> value.

bits 64
default rel

section .text
global _start
_start:
    ; task count
    mov  edi, 1
    mov  eax, 61
    int  0x80
    test rax, rax
    jz   .fail
    ; free frames
    mov  edi, 2
    mov  eax, 61
    int  0x80
    test rax, rax
    jz   .fail
    ; uptime advances
    mov  edi, 3
    mov  eax, 61
    int  0x80
    mov  r14, rax
    mov  rcx, 50000000
.spin:
    dec  rcx
    jnz  .spin
    mov  edi, 3
    mov  eax, 61
    int  0x80
    cmp  rax, r14
    jbe  .fail
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
okmsg:   db "SYSINFOPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "SYSINFOPROBE: FAIL", 10
failmsg_len equ $ - failmsg
