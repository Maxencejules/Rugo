; base-shell: the first real external program executed from the package
; store on disk. Prints one marker over sys_debug_write and exits cleanly.
; Linked at the exec app window (0x140_0000) - see linker.ld.

bits 64
default rel

section .text
global _start
_start:
    lea  rdi, [rel msg]
    mov  esi, msg_len
    xor  eax, eax            ; sys_debug_write (id 0)
    int  0x80
    mov  eax, 2              ; sys_thread_exit (id 2)
    int  0x80
.hang:
    jmp  .hang

section .rodata
msg: db "BASESH: hello from disk", 10
msg_len equ $ - msg
