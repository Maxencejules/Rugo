; dlprobe: dynamic-loading proof (sys_dlctl = dlopen/dlsym + call).
;
; dlopen("dlmod") -> module base; dlsym("addone") -> function VA; call it with
; arg 41 and expect 42 -> proof the kernel loaded a separately-authored module
; into an executable user region, resolved a symbol, and the app executed it.
; sys_dlctl: rdi=op (1=dlopen, 2=dlsym), rsi=name ptr. Returns VA or -1.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 1                 ; op 1 = dlopen
    lea  rsi, [rel modname]
    mov  eax, 60               ; sys_dlctl
    int  0x80
    cmp  rax, -1
    je   .fail
    ; second dlopen of the same module must also succeed (idempotent)
    mov  edi, 1
    lea  rsi, [rel modname]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  edi, 2                 ; op 2 = dlsym
    lea  rsi, [rel symname]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    ; call the resolved function: addone(41) must return 42
    mov  rdi, 41
    call rax
    cmp  rax, 42
    jne  .fail
    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax              ; syscall 0 = write
    int  0x80
    mov  eax, 2                ; exit
    int  0x80
.fail:
    lea  rdi, [rel failmsg]
    mov  esi, failmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
modname: db "dlmod", 0, 0, 0                 ; 8 bytes, null-padded
symname: db "addone", 0, 0, 0, 0, 0, 0       ; 12 bytes, null-padded
okmsg:   db "DLPROBE: dlsym ok", 10
okmsg_len equ $ - okmsg
failmsg: db "DLPROBE: FAIL", 10
failmsg_len equ $ - failmsg
