; devprobe: /dev character-device proof (pseudo-fs, full-os guide II.5).
;
; Opens /dev/zero (reads must be all-zero), /dev/urandom (reads must vary
; from zero), and /dev/null (writes are accepted and discarded). open=18,
; read=19, write=20; RDONLY=0, WRONLY=1.

bits 64
default rel

section .text
global _start
_start:
    ; /dev/zero, RDONLY
    lea  rdi, [rel pzero]
    xor  esi, esi
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax
    mov  rdi, r12
    lea  rsi, [rel buf]
    mov  edx, 16
    mov  eax, 19
    int  0x80
    cmp  rax, 16
    jne  .fail
    ; buf must be all zero
    lea  rbx, [rel buf]
    xor  al, al
    mov  rcx, 16
.orz:
    or   al, [rbx]
    inc  rbx
    dec  rcx
    jnz  .orz
    test al, al
    jnz  .fail

    ; /dev/urandom, RDONLY
    lea  rdi, [rel purandom]
    xor  esi, esi
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  rdi, rax
    lea  rsi, [rel buf]
    mov  edx, 16
    mov  eax, 19
    int  0x80
    cmp  rax, 16
    jne  .fail
    ; buf must be non-zero
    lea  rbx, [rel buf]
    xor  al, al
    mov  rcx, 16
.oru:
    or   al, [rbx]
    inc  rbx
    dec  rcx
    jnz  .oru
    test al, al
    jz   .fail

    ; /dev/null, WRONLY
    lea  rdi, [rel pnull]
    mov  esi, 1
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  rdi, rax
    lea  rsi, [rel buf]
    mov  edx, 8
    mov  eax, 20
    int  0x80
    cmp  rax, 8
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
pzero:    db "/dev/zero", 0
purandom: db "/dev/urandom", 0
pnull:    db "/dev/null", 0
okmsg:    db "DEVPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg:  db "DEVPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
buf: resb 16
