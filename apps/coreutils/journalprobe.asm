; journalprobe: FS journaling proof (full-os guide Part II.5).
;
; sys_sysinfo(op=10, id 61) logs a write to the journal, verifies the target is
; un-applied (crash-before-apply), replays the journal, and confirms the target
; now holds the logged data -> 1 on success. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 10
    mov  eax, 61
    int  0x80
    cmp  rax, 1
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
okmsg:   db "JOURNALPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "JOURNALPROBE: FAIL", 10
failmsg_len equ $ - failmsg
