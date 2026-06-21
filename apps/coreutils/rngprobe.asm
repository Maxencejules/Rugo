; rngprobe: CSPRNG proof (sys_getrandom, id 54).
;
; Fills two 16-byte buffers from getrandom and proves (a) the bytes are not
; all zero (the pool produced output) and (b) the two draws differ (the
; pool advances). sys_getrandom: rdi=buf, rsi=len -> bytes written.

bits 64
default rel

section .text
global _start
_start:
    ; getrandom(buf1, 16)
    lea  rdi, [rel buf1]
    mov  esi, 16
    mov  eax, 54
    int  0x80
    cmp  rax, 16
    jne  .fail
    ; OR all 16 bytes of buf1 - must be non-zero
    lea  rbx, [rel buf1]
    xor  al, al
    mov  rcx, 16
.or1:
    or   al, [rbx]
    inc  rbx
    dec  rcx
    jnz  .or1
    test al, al
    jz   .fail
    ; getrandom(buf2, 16)
    lea  rdi, [rel buf2]
    mov  esi, 16
    mov  eax, 54
    int  0x80
    cmp  rax, 16
    jne  .fail
    ; buf1 vs buf2 must differ somewhere
    lea  rsi, [rel buf1]
    lea  rdi, [rel buf2]
    mov  rcx, 16
    xor  r8b, r8b
.cmp:
    mov  al, [rsi]
    cmp  al, [rdi]
    je   .eq
    mov  r8b, 1
.eq:
    inc  rsi
    inc  rdi
    dec  rcx
    jnz  .cmp
    test r8b, r8b
    jz   .fail
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
okmsg:   db "RNGPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "RNGPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
buf1: resb 16
buf2: resb 16
