; multidlprobe: POSIX dlopen HANDLE TABLE proof (sys_dlctl op4 = dlsym_h).
;
; The single-slot loader could keep only ONE object live at a time. This proves
; the handle table: dlopen("libdl") TWICE yields two concurrently-live objects at
; two different randomized bases; op4 dlsym_h(handle, "getval") resolves the SAME
; symbol against each SPECIFIC handle, giving two DISTINCT VAs that are each
; callable (each copy independently relocated -> 42). dlclose(h1) then frees only
; that object: h2's dlsym_h still resolves + runs, while h1's now returns -1.
;
; sys_dlctl: rdi=op, rsi=a2, rdx=a3.  op1 dlopen(name)->handle/base,
;   op3 dlclose(handle)->0, op4 dlsym_h(handle, name)->VA. Returns -1 on error.

bits 64
default rel

section .text
global _start
_start:
    ; --- dlopen("libdl") #1 -> h1 (base1) ---
    mov  edi, 1
    lea  rsi, [rel modname]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax                 ; r12 = h1 (callee-saved across int 0x80 + call)

    ; --- dlopen("libdl") #2 -> h2 (base2) ---
    mov  edi, 1
    lea  rsi, [rel modname]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r13, rax                 ; r13 = h2
    cmp  r12, r13                 ; two LIVE objects must sit at different bases
    je   .fail

    ; --- dlsym_h(h1,"getval") -> va1 ; dlsym_h(h2,"getval") -> va2 ---
    mov  edi, 4
    mov  rsi, r12                 ; handle h1
    lea  rdx, [rel sym_getval]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r14, rax                 ; r14 = va1 (= base1 + off)

    mov  edi, 4
    mov  rsi, r13                 ; handle h2
    lea  rdx, [rel sym_getval]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r15, rax                 ; r15 = va2 (= base2 + off)

    cmp  r14, r15                 ; per-handle resolution: distinct concurrent VAs
    je   .fail

    ; --- both copies callable concurrently (each relocated -> 42) ---
    mov  rax, r14
    call rax                      ; getval() in h1
    cmp  rax, 42
    jne  .fail
    mov  rax, r15
    call rax                      ; getval() in h2
    cmp  rax, 42
    jne  .fail

    ; --- dlclose(h1): frees ONLY h1; h2 stays live ---
    mov  edi, 3
    mov  rsi, r12                 ; handle h1
    mov  eax, 60
    int  0x80
    cmp  rax, 0
    jne  .fail

    ; --- h2 survives the close: dlsym_h still resolves + runs ---
    mov  edi, 4
    mov  rsi, r13                 ; handle h2
    lea  rdx, [rel sym_getval]
    mov  eax, 60
    int  0x80
    cmp  rax, -1
    je   .fail
    call rax                      ; getval() in h2 -> still 42
    cmp  rax, 42
    jne  .fail

    ; --- h1 is gone: its handle no longer resolves ---
    mov  edi, 4
    mov  rsi, r12                 ; handle h1 (closed)
    lea  rdx, [rel sym_getval]
    mov  eax, 60
    int  0x80
    cmp  rax, -1                  ; MUST be -1 (closed handle)
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
okmsg:   db "MULTIDL: handle table ok", 10
okmsg_len equ $ - okmsg
failmsg: db "MULTIDL: FAIL", 10
failmsg_len equ $ - failmsg
