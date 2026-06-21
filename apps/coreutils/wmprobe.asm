; wmprobe: standing window-server PERSISTENT surface registry proof (sys_ioctl 56).
;
; Unlike op 4 (a one-shot composite of a throwaway list), the registry persists
; across calls and is owner-stamped, so this proves the window-server data model:
;   op 8 wm_register(slot, desc) -> register a persistent owned surface
;   op 9 wm_compose()            -> composite the WHOLE registry in z-order -> count
;   op 10 wm_clear(slot)         -> remove the caller's surface (owner-checked)
; It registers two windows (red z=0, blue z=1 on top), composes (=2), clears one,
; composes (=1), then EXITS leaving the other window registered -- so wmcheck can
; observe the kernel's exit-cleanup (a dead client's windows disappear).
;
; sys_ioctl: rdi=op, rsi=a2, rdx=a3 (id 56).

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 8                 ; wm_register slot 0 (red, z=0)
    xor  esi, esi
    lea  rdx, [rel desc0]
    mov  eax, 56
    int  0x80
    cmp  rax, 0
    jne  .fail

    mov  edi, 8                 ; wm_register slot 1 (blue, z=1, overlaps)
    mov  esi, 1
    lea  rdx, [rel desc1]
    mov  eax, 56
    int  0x80
    cmp  rax, 1
    jne  .fail

    mov  edi, 9                 ; wm_compose -> both persistent surfaces
    mov  eax, 56
    int  0x80
    cmp  rax, 2                 ; registry held + composited 2 windows
    jne  .fail

    mov  edi, 10                ; wm_clear slot 0 (owner-checked)
    xor  esi, esi
    mov  eax, 56
    int  0x80
    cmp  rax, 0
    jne  .fail

    mov  edi, 9                 ; wm_compose again
    mov  eax, 56
    int  0x80
    cmp  rax, 1                 ; one window left after the clear
    jne  .fail

    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax
    int  0x80
    ; exit WITHOUT clearing slot 1: the kernel must clean it up on exit.
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
; descriptor = [geom: x<<48|y<<32|w<<16|h] [color(low32)|z(high32)]
desc0: dq 0x000A000A00140014, 0x0000000000FF0000   ; (10,10,20,20) red   z=0
desc1: dq 0x000F000F00140014, 0x00000001000000FF   ; (15,15,20,20) blue  z=1
okmsg:   db "WM: registry compose=2 afterclear=1 ok", 10
okmsg_len equ $ - okmsg
failmsg: db "WM: FAIL", 10
failmsg_len equ $ - failmsg
