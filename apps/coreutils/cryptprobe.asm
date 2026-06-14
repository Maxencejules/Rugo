; cryptprobe: disk-encryption proof (full-os guide Part IV.10).
;
; sys_sysinfo(op=9, id 61) encrypts a known plaintext, writes it to a scratch
; sector, reads it back raw (ciphertext != plaintext), decrypts, and verifies
; the round trip -> 1 on success. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 9
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
okmsg:   db "CRYPTPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "CRYPTPROBE: FAIL", 10
failmsg_len equ $ - failmsg
