; userprobe: multi-user uid privilege proof (full-os guide Part IV.10).
;
; sys_proc_ctl (id 51): op 3 = getuid, op 4 = setuid(rsi). An external app runs
; as uid 100, so getuid returns 100 and setuid is denied (only root may change
; uid); the audit log records the denial. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    ; getuid -> 100
    mov  edi, 3
    mov  eax, 51
    int  0x80
    cmp  rax, 100
    jne  .fail
    ; setuid(0) must be DENIED for a non-root caller
    mov  edi, 4
    xor  esi, esi
    mov  eax, 51
    int  0x80
    cmp  rax, -1
    jne  .fail
    ; uid is unchanged
    mov  edi, 3
    mov  eax, 51
    int  0x80
    cmp  rax, 100
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
okmsg:   db "USERPROBE: uid=100 setuid-denied ok", 10
okmsg_len equ $ - okmsg
failmsg: db "USERPROBE: FAIL", 10
failmsg_len equ $ - failmsg
