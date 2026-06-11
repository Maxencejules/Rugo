; nxprobe: W^X proof. Copies a `ret` instruction onto the stack (a
; demand-paged, NX stack stride) and calls it. With NX enforced the
; fetch faults (USERPF err bit 4 set) and the kernel kills this task;
; reaching .alive means W^X is broken.

bits 64
default rel

section .text
global _start
_start:
    lea  rdi, [rel pfx]
    mov  esi, pfx_len
    xor  eax, eax            ; sys_debug_write
    int  0x80
    sub  rsp, 16
    mov  byte [rsp], 0xC3    ; ret
    lea  rax, [rsp]
    call rax                 ; must fault here
.alive:
    lea  rdi, [rel bad]
    mov  esi, bad_len
    xor  eax, eax
    int  0x80
    mov  eax, 2              ; sys_thread_exit
    int  0x80
.hang:
    jmp  .hang

section .rodata
pfx: db "NXPROBE: jumping to stack", 10
pfx_len equ $ - pfx
bad: db "NXPROBE: executed from stack (W^X broken)", 10
bad_len equ $ - bad
