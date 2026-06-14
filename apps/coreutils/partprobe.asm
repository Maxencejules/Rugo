; partprobe: MBR partition-table proof (full-os guide Part II.5 partitions).
;
; sys_sysinfo(op=5, id 61) reads LBA 0, validates the 0x55AA signature, logs
; each non-empty primary partition ("PART: ..."), and returns the count.
; console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 5
    mov  eax, 61
    int  0x80
    cmp  rax, -1
    je   .fail
    test rax, rax
    jz   .fail                 ; zero partitions -> fail
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
okmsg:   db "PARTPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "PARTPROBE: FAIL", 10
failmsg_len equ $ - failmsg
