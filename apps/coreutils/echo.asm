; echo: print the spawn argument string. Entry: rdi = args, rsi = len.

bits 64
default rel

section .text
global _start
_start:
    test rsi, rsi
    jz   .nl
    xor  eax, eax            ; sys_debug_write
    int  0x80
.nl:
    lea  rdi, [rel nl]
    mov  esi, 1
    xor  eax, eax
    int  0x80
    mov  eax, 2              ; sys_thread_exit
    int  0x80
.hang:
    jmp  .hang

section .rodata
nl: db 10
