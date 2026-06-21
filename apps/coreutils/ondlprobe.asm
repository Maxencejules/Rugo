; ondlprobe: load a shared object FROM THE FILESYSTEM (sys_dlctl dlopen of a path).
;
; The kernel seeds /data/dltest.so (a copy of the embedded libdl) at boot.
; dlopen("/data/dltest.so") reads it off the VFS and links it; dlsym + call
; proves on-disk .so loading works end to end (getval()==42 requires its
; R_X86_64_RELATIVE relocation to have been applied on the on-disk image too).
; sys_dlctl: rdi=op (1=dlopen, 2=dlsym), rsi=name ptr. Returns VA or -1.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 1                 ; dlopen("/data/dltest.so")
    lea  rsi, [rel modpath]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
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
    mov  edi, 2                 ; dlsym("getval")
    lea  rsi, [rel sym_getval]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    call rax                    ; getval() -> 42 iff the on-disk image was relocated
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
; 32-byte path buffer (the kernel dlopen reads up to 32 bytes of the name).
modpath:    db "/data/dltest.so", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
sym_addtwo: db "addtwo", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0      ; 16 bytes
sym_getval: db "getval", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0      ; 16 bytes
okmsg:   db "ONDISKDL: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "ONDISKDL: FAIL", 10
failmsg_len equ $ - failmsg
