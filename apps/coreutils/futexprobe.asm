; futexprobe: clone + futex proof (sys_proc_ctl clone op 2, sys_futex 52).
;
; The parent clones a thread (shared address space), then blocks in
; futex_wait on a shared word. The child sets the word and wakes the parent.
; Proves: clone shares the address space (same VA -> same memory), and futex
; wait/wake hand off correctly.
;
; clone: rdi=2 (op), rsi=entry, eax=51. futex: rdi=op, rsi=uaddr, rdx=val,
; eax=52 (op 1 wait, op 2 wake).

bits 64
default rel

section .text
global _start
_start:
    ; clone a thread starting at child_entry
    mov  edi, 2
    lea  rsi, [rel child_entry]
    mov  eax, 51
    int  0x80
    cmp  rax, -1
    je   .fail
    ; futex_wait(&futexvar, 0) - blocks until the child wakes us
    mov  edi, 1
    lea  rsi, [rel futexvar]
    xor  edx, edx
    mov  eax, 52
    int  0x80
    ; resumed: the shared word must now be 1
    cmp  dword [rel futexvar], 1
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

child_entry:
    mov  dword [rel futexvar], 1
    mov  edi, 2                 ; futex wake
    lea  rsi, [rel futexvar]
    mov  edx, 1
    mov  eax, 52
    int  0x80
    mov  eax, 2                 ; thread_exit
    int  0x80

section .data
futexvar: dd 0
okmsg:    db "FUTEXPROBE: woken ok", 10
okmsg_len equ $ - okmsg
failmsg:  db "FUTEXPROBE: FAIL", 10
failmsg_len equ $ - failmsg
