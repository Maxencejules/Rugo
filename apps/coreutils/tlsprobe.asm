; tlsprobe: thread-local storage proof (sys_vm_ctl op 5 = set_tls, ABI id 50).
;
; Sets the task's %fs base to a TLS block, then proves FS-relative addressing
; works and that the base is per-task (restored across a context switch):
;   set_tls(&tls_buf)         -> %fs base = &tls_buf
;   mov [fs:0], magic         -> writes tls_buf[0] via the segment base
;   read tls_buf[0] directly  -> equals magic (so fs:0 == &tls_buf)
;   yield (force a switch away + back), re-read [fs:0] -> still magic
; The last step only holds if the kernel restored THIS task's fs.base on resume
; (r4_switch_to), i.e. TLS is per-task. Fails unless every check matches.
;
; sys_vm_ctl: rdi=op (5=set_tls), rsi=a2 (base) (id 50). sys_yield: id 3.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 5                  ; set_tls
    lea  rsi, [rel tls_buf]
    mov  eax, 50                 ; sys_vm_ctl
    int  0x80
    cmp  rax, 0
    jne  .fail

    mov  rax, 0x5A5A5A5A5A5A5A5A  ; magic
    mov  [fs:0], rax             ; tls_buf[0] = magic, via the %fs base

    mov  rbx, [rel tls_buf]      ; read the same slot directly
    mov  rdx, 0x5A5A5A5A5A5A5A5A
    cmp  rbx, rdx                ; fs:0 must alias &tls_buf
    jne  .fail

    mov  eax, 3                  ; sys_yield -> run other tasks (their fs.base=0)
    int  0x80

    mov  rcx, [fs:0]             ; re-read via %fs after the context switch
    cmp  rcx, rdx                ; must survive (this task's fs.base was restored)
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
tls_buf: dq 0, 0, 0, 0, 0, 0, 0, 0   ; 64-byte TLS block
okmsg:   db "TLS: fs-base tls ok", 10
okmsg_len equ $ - okmsg
failmsg: db "TLS: FAIL", 10
failmsg_len equ $ - failmsg
