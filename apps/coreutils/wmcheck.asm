; wmcheck: prove the window server's per-client EXIT-CLEANUP lifecycle.
;
; wmprobe (run first) registered a surface in slot 1 and exited WITHOUT clearing
; it. A real window server removes a dead client's windows, so by the time this
; second, DIFFERENT client runs, wm_compose must paint 0 surfaces -- the exited
; owner's window is gone. (If cleanup were missing, the stale window would still
; composite and this would see 1.)
;
; sys_ioctl: rdi=op (9 = wm_compose), id 56.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 9                 ; wm_compose the persistent registry
    mov  eax, 56
    int  0x80
    cmp  rax, 0                 ; the exited client's window must be gone
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
okmsg:   db "WMCHECK: after-owner-exit=0 ok", 10
okmsg_len equ $ - okmsg
failmsg: db "WMCHECK: FAIL", 10
failmsg_len equ $ - failmsg
