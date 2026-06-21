; compositorprobe: window-server compose proof (sys_ioctl op 4 = compose).
;
; Submits two surfaces to the kernel compositor: a large BLUE background (z=0)
; and a smaller RED window (z=1) fully inside it. The kernel sorts by z and
; blits background-first, window-last, so a QMP screendump shows the red window
; ON TOP of the blue background (both visible) — proof of z-ordered composition.
;
; sys_ioctl: rdi=op(4), rsi=ptr to surface array, rdx=count. Each surface is two
; u64s: [x<<48|y<<32|w<<16|h] then [color(low32)|z(high32)]. Returns blitted count.

bits 64
default rel

section .text
global _start
_start:
    lea  rsi, [rel surfaces]
    mov  edi, 4                    ; op 4 = compose
    mov  edx, 2                    ; two surfaces
    mov  eax, 56                   ; sys_ioctl
    int  0x80
    cmp  rax, 2                    ; both surfaces must have been blitted
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
align 8
surfaces:
    dq 0x006400640190012C         ; bg:  x=100 y=100 w=400 h=300
    dq 0x00000000000000FF         ; bg:  color=blue(0x0000FF) z=0
    dq 0x00C8009600C80096         ; win: x=200 y=150 w=200 h=150
    dq 0x0000000100FF0000         ; win: color=red(0xFF0000) z=1
okmsg:   db "COMPOSITORPROBE: compose ok", 10
okmsg_len equ $ - okmsg
failmsg: db "COMPOSITORPROBE: FAIL", 10
failmsg_len equ $ - failmsg
