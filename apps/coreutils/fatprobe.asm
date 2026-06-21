; fatprobe: FAT16 file-read proof (full-os guide Part II.5 filesystem maturity).
;
; sys_sysinfo(op=6, id 61) reads the file HELLO.TXT from a FAT volume on the
; block device into the supplied buffer. The probe echoes the contents so the
; test can assert them. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 6
    lea  rsi, [rel buf]
    mov  edx, 64
    mov  eax, 61
    int  0x80
    cmp  rax, -1
    je   .fail
    test rax, rax
    jz   .fail
    ; echo the file contents
    lea  rdi, [rel buf]
    mov  esi, eax
    xor  eax, eax
    int  0x80
    ; mark a clean newline + verdict
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
okmsg:   db 10, "FATPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "FATPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
buf: resb 64
