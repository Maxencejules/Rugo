; cat: print a file from the /data tree. Entry: rdi = path (NUL
; terminated by the kernel), rsi = len.

bits 64
default rel

section .text
global _start
_start:
    test rsi, rsi
    jz   .err
    ; open(path, RDONLY=0, 0)
    xor  esi, esi
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .err
    mov  r12, rax            ; fd
.loop:
    mov  rdi, r12
    lea  rsi, [rel buf]
    mov  edx, 192
    mov  eax, 19             ; sys_read
    int  0x80
    cmp  rax, -1
    je   .close_err
    test rax, rax
    jz   .done
    ; debug_write(buf, n)
    lea  rdi, [rel buf]
    mov  rsi, rax
    xor  eax, eax
    int  0x80
    jmp  .loop
.done:
    mov  rdi, r12
    mov  eax, 21             ; sys_close
    int  0x80
    mov  eax, 2
    int  0x80
.close_err:
    mov  rdi, r12
    mov  eax, 21
    int  0x80
.err:
    lea  rdi, [rel emsg]
    mov  esi, emsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.hang:
    jmp  .hang

section .rodata
emsg: db "cat: error", 10
emsg_len equ $ - emsg

section .bss
buf: resb 192
