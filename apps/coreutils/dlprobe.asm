; dlprobe: real ELF .so dynamic-loading proof (sys_dlctl = dlopen/dlsym + call).
;
; dlopen("libdl") loads the kernel-embedded ELF shared object; dlsym("getval")
; resolves a function whose result depends on an R_X86_64_RELATIVE relocation
; (returns 42 only if the loader applied it); dlsym("addtwo") + call(40) == 42
; proves symbol resolution + ring-3 execution of the loaded code.
; sys_dlctl: rdi=op (1=dlopen, 2=dlsym), rsi=name ptr. Returns VA or -1.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 1                 ; dlopen
    lea  rsi, [rel modname]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  edi, 2                 ; dlsym("getval")
    lea  rsi, [rel sym_getval]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    call rax                    ; getval() -> 42 iff the relocation was applied
    cmp  rax, 42
    jne  .fail
    mov  edi, 2                 ; dlsym("addtwo")
    lea  rsi, [rel sym_addtwo]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  rdi, 40
    call rax                    ; addtwo(40) -> 42
    cmp  rax, 42
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
modname:    db "libdl", 0, 0, 0                            ; 8 bytes, null-padded
sym_getval: db "getval", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0      ; 16 bytes
sym_addtwo: db "addtwo", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0      ; 16 bytes
okmsg:   db "DLPROBE: dlsym ok", 10
okmsg_len equ $ - okmsg
failmsg: db "DLPROBE: FAIL", 10
failmsg_len equ $ - failmsg
