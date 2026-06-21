; surfprobe: composite a real per-pixel SURFACE to the framebuffer (sys_ioctl
; op 6). Builds a 32x32 ARGB bitmap on the stack -- top half GREEN, bottom half
; BLUE -- and asks the kernel to blit it at (300,200). A solid-color rect (op 4)
; could not produce a two-color bitmap, so a screendump showing BOTH colors in
; the region proves per-pixel client surfaces. console-write=0, exit=2.

bits 64
default rel

section .text
global _start
_start:
    sub  rsp, 4096              ; 1024-pixel (32x32) ARGB buffer on the stack
    xor  rcx, rcx
.fill:
    mov  eax, 0x0000FF00        ; green (upper half: pixels 0..511 = rows 0..15)
    cmp  rcx, 512
    jb   .put
    mov  eax, 0x000000FF        ; blue (lower half)
.put:
    mov  [rsp + rcx*4], eax
    inc  rcx
    cmp  rcx, 1024
    jb   .fill
    ; surface_compose: ioctl(op=6, pixels=rsp, geom = x300 y200 w32 h32)
    mov  edi, 6
    mov  rsi, rsp
    mov  rdx, 0x012C00C800200020
    mov  eax, 56
    int  0x80
    cmp  rax, -1
    je   .fail
    add  rsp, 4096
    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.fail:
    add  rsp, 4096
    lea  rdi, [rel failmsg]
    mov  esi, failmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
okmsg:   db "SURFACE: compose ok", 10
okmsg_len equ $ - okmsg
failmsg: db "SURFACE: compose FAIL", 10
failmsg_len equ $ - failmsg
