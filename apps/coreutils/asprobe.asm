; asprobe: per-process address-space isolation + concurrency probe.
;
; Spawned concurrently as "asprobe A" and "asprobe B". Each instance is
; the SAME ELF, so its `slot` global lives at the SAME virtual address in
; both. Each writes an id-derived value there, yields repeatedly so the
; sibling interleaves, then proves the global still holds its OWN value.
;
; With per-process address spaces that VA maps to a different physical
; frame in each instance, so both pass ("iso ok"). Under a shared address
; space the second writer would clobber the first's frame and one of them
; would read the wrong value ("iso FAIL"). The interleaved "tick" markers
; from both ids prove the two apps are resident and running concurrently.

bits 64
default rel

section .text
global _start
_start:
    ; rdi = args ptr, rsi = args len. First byte is the id (default 'A').
    mov  al, 'A'
    test rsi, rsi
    jz   .haveid
    mov  al, [rdi]
.haveid:
    mov  [rel myid], al
    ; value = id byte broadcast across all 8 bytes
    movzx rbx, al
    mov  rcx, 0x0101010101010101
    imul rbx, rcx
    mov  [rel slot], rbx

    mov  r12, 8                 ; yield/tick rounds
.loop:
    mov  al, [rel myid]
    mov  byte [rel tickmsg + tick_id_off], al
    lea  rdi, [rel tickmsg]
    mov  esi, tickmsg_len
    xor  eax, eax               ; sys_debug_write
    int  0x80
    mov  eax, 3                 ; sys_yield
    int  0x80
    dec  r12
    jnz  .loop

    ; isolation check: slot must still equal our own value
    movzx rbx, byte [rel myid]
    mov  rcx, 0x0101010101010101
    imul rbx, rcx
    cmp  [rel slot], rbx
    jne  .fail

    mov  al, [rel myid]
    mov  byte [rel okmsg + ok_id_off], al
    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2                 ; sys_thread_exit
    int  0x80

.fail:
    mov  al, [rel myid]
    mov  byte [rel failmsg + fail_id_off], al
    lea  rdi, [rel failmsg]
    mov  esi, failmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
slot:    dq 0
tickmsg: db "ASPROBE: tick id=_", 10
tickmsg_len equ $ - tickmsg
tick_id_off equ tickmsg_len - 2
okmsg:   db "ASPROBE: iso ok id=_", 10
okmsg_len equ $ - okmsg
ok_id_off equ okmsg_len - 2
failmsg: db "ASPROBE: iso FAIL id=_", 10
failmsg_len equ $ - failmsg
fail_id_off equ failmsg_len - 2

section .bss
myid: resb 1
