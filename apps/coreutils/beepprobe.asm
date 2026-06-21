; beepprobe: PC speaker proof (full-os guide Part III audio).
;
; sys_ioctl(op=3, id 56) programs PIT channel 2 to a tone and gates the speaker
; on, returning the read-back gate bits (3 = enabled). console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 3                 ; op 3 = beep
    mov  esi, 440              ; frequency (Hz)
    mov  eax, 56               ; sys_ioctl
    int  0x80
    cmp  rax, 3                ; speaker gate+data bits enabled?
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
okmsg:   db "BEEPPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "BEEPPROBE: FAIL", 10
failmsg_len equ $ - failmsg
