; dlprobe: real ELF .so dynamic-loading + code-base ASLR proof (sys_dlctl).
;
; dlopen("libdl") loads the kernel-embedded ELF shared object at a RANDOMIZED base
; (code-base ASLR); doing it twice yields two different load bases -- proof the base
; is randomized per load. dlsym("getval") resolves a function whose result depends on
; an R_X86_64_RELATIVE relocation applied at the (random) base; addtwo/getgvar/callsum
; exercise plain/GLOB_DAT/JUMP_SLOT resolution + ring-3 execution there.
; sys_dlctl: rdi=op (1=dlopen, 2=dlsym), rsi=name ptr. Returns VA or -1.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 1                 ; dlopen #1 -> base1
    lea  rsi, [rel modname]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax               ; save base1 (preserved across the next int 0x80)
    mov  edi, 1                 ; dlopen #2 -> base2 (a fresh randomized base)
    lea  rsi, [rel modname]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    cmp  r12, rax               ; code-base ASLR: the two load bases MUST differ
    je   .fail
    mov  edi, 2                 ; dlsym("getval")  (resolves against the current base)
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
    mov  edi, 2                 ; dlsym("getgvar")
    lea  rsi, [rel sym_getgvar]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    call rax                    ; getgvar() -> 99 iff R_X86_64_GLOB_DAT was applied
    cmp  rax, 99
    jne  .fail
    mov  edi, 2                 ; dlsym("callsum")
    lea  rsi, [rel sym_callsum]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    call rax                    ; callsum() -> extadd(40)=42 iff R_X86_64_JUMP_SLOT applied
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
sym_getgvar: db "getgvar", 0, 0, 0, 0, 0, 0, 0, 0, 0       ; 16 bytes
sym_callsum: db "callsum", 0, 0, 0, 0, 0, 0, 0, 0, 0       ; 16 bytes
okmsg:   db "DLPROBE: aslr+dlsym ok", 10
okmsg_len equ $ - okmsg
failmsg: db "DLPROBE: FAIL", 10
failmsg_len equ $ - failmsg
