; fatbigprobe: FAT16 multi-cluster (FAT-chain) read proof (full-os Part II.5).
;
; sys_sysinfo(op=12, id 61) reads BIG.TXT by walking the FAT cluster chain and
; verifies a deterministic pattern across the cluster boundary inside the kernel,
; emitting "FATBIG: chain read ok size=...". This probe just triggers it and
; reports a verdict. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 12
    xor  esi, esi
    xor  edx, edx
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
okmsg:   db "FATBIGPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "FATBIGPROBE: FAIL", 10
failmsg_len equ $ - failmsg
