; gfxprobe: framebuffer graphics proof (sys_ioctl op 1 = fb blit).
;
; Draws a 240x180 red rectangle at (200,150). sys_ioctl: rdi=op,
; rsi=packed rect (x<<48 | y<<32 | w<<16 | h, each u16), rdx=XRGB color.
; A QMP screendump then shows thousands of red pixels.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 1
    mov  rsi, 0x00C8009600F000B4   ; x=200 y=150 w=240 h=180
    mov  rdx, 0x00FF0000           ; red
    mov  eax, 56
    int  0x80
    cmp  rax, 0
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
okmsg:   db "GFXPROBE: blit ok", 10
okmsg_len equ $ - okmsg
failmsg: db "GFXPROBE: FAIL", 10
failmsg_len equ $ - failmsg
