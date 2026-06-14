; fatlsprobe: FAT16 directory-listing proof (full-os guide Part II.5).
;
; sys_sysinfo(op=8, id 61) lists the FAT16 root directory (kernel logs each
; "FATLS: <name> size=0x..") and returns the entry count. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 8
    mov  eax, 61
    int  0x80
    cmp  rax, -1
    je   .fail
    test rax, rax
    jz   .fail
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
okmsg:   db "FATLSPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "FATLSPROBE: FAIL", 10
failmsg_len equ $ - failmsg
