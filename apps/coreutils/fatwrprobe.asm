; fatwrprobe: FAT16 file-WRITE proof (full-os guide Part II.5 filesystem maturity).
;
; sys_sysinfo(op=11, id 61) writes a single-cluster file to the FAT volume on the
; block device, then reads it back and verifies a byte-exact round-trip, printing
; "FATWR: write+read ok" on success. The probe just triggers it and reports the
; verdict (returns 1 on success, 0 on failure). console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 11               ; op 11 = FAT16 write self-test
    xor  esi, esi
    xor  edx, edx
    mov  eax, 61               ; sys_sysinfo
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
okmsg:   db "FATWRPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "FATWRPROBE: FAIL", 10
failmsg_len equ $ - failmsg
