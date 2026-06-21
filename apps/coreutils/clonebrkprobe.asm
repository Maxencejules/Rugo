; clonebrkprobe: proves the program break (brk) is per-ADDRESS-SPACE across
; clone (sys_proc_ctl op2) threads -- POSIX semantics. The kernel keeps every
; thread sharing one pml4 at the same break via two mechanisms:
;   * copy-at-clone: a new thread starts at the caller's CURRENT break;
;   * propagate-on-write: any thread's brk change is reflected to all siblings.
; The clone thread checks it INHERITED the parent's grown break, then grows the
; shared break further; the parent (woken via futex) checks it SEES that growth.
; A cloned thread starts with zeroed registers, so it reads the base from a
; shared .data global -- clone shares the address space, so .data is shared
; memory (same trick futexprobe uses for its futex word).
;
; clone: edi=2, rsi=entry, eax=51. brk: edi=3, rsi=new (0 queries), eax=50.
; futex: edi=op (1 wait / 2 wake), rsi=uaddr, edx=val, eax=52. write=eax=0,
; exit=eax=2. brk window base = VM_BRK_BASE (0x0100_0000).

bits 64
default rel

section .text
global _start
_start:
    ; main: base = brk(0); publish it; grow the break to base + 0x3000
    mov  edi, 3
    xor  esi, esi
    mov  eax, 50
    int  0x80
    mov  [rel g_base], rax
    lea  rsi, [rax + 0x3000]
    mov  edi, 3
    mov  eax, 50
    int  0x80
    ; clone a thread at .clonefn (it SHARES this address space)
    mov  edi, 2
    lea  rsi, [rel .clonefn]
    mov  eax, 51
    int  0x80
    cmp  rax, -1
    je   .cloneerr
    ; block until the clone has checked + grown the shared break
    mov  edi, 1                 ; futex wait(&g_done, 0)
    lea  rsi, [rel g_done]
    xor  edx, edx
    mov  eax, 52
    int  0x80
    ; main must now SEE the clone's grown break: base + 0x5000
    mov  edi, 3
    xor  esi, esi
    mov  eax, 50
    int  0x80
    mov  r13, [rel g_base]
    lea  rcx, [r13 + 0x5000]
    cmp  rax, rcx
    jne  .mainfail
    lea  rdi, [rel mokmsg]
    mov  esi, mokmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.mainfail:
    lea  rdi, [rel mfailmsg]
    mov  esi, mfailmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.cloneerr:
    lea  rdi, [rel cerrmsg]
    mov  esi, cerrmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

; ---- cloned thread: fresh registers, state read from the shared AS ----
.clonefn:
    ; brk(0) must equal the INHERITED break (g_base + 0x3000)
    mov  edi, 3
    xor  esi, esi
    mov  eax, 50
    int  0x80
    mov  r13, [rel g_base]
    lea  rcx, [r13 + 0x3000]
    cmp  rax, rcx
    jne  .cfail
    lea  rdi, [rel cokmsg]
    mov  esi, cokmsg_len
    xor  eax, eax
    int  0x80
    ; grow the SHARED break to g_base + 0x5000
    mov  r13, [rel g_base]
    lea  rsi, [r13 + 0x5000]
    mov  edi, 3
    mov  eax, 50
    int  0x80
.cwake:
    ; signal main and wake it
    mov  dword [rel g_done], 1
    mov  edi, 2                 ; futex wake(&g_done, 1)
    lea  rsi, [rel g_done]
    mov  edx, 1
    mov  eax, 52
    int  0x80
    mov  eax, 2
    int  0x80
.cfail:
    lea  rdi, [rel cfailmsg]
    mov  esi, cfailmsg_len
    xor  eax, eax
    int  0x80
    jmp  .cwake                 ; still wake main so it does not hang

section .data
g_base:    dq 0
g_done:    dd 0
cokmsg:    db "CLONEBRK: clone inherited ok", 10
cokmsg_len equ $ - cokmsg
cfailmsg:  db "CLONEBRK: clone inherited FAIL", 10
cfailmsg_len equ $ - cfailmsg
mokmsg:    db "CLONEBRK: main saw-shared ok", 10
mokmsg_len equ $ - mokmsg
mfailmsg:  db "CLONEBRK: main saw-shared FAIL", 10
mfailmsg_len equ $ - mfailmsg
cerrmsg:   db "CLONEBRK: FAIL clone-err", 10
cerrmsg_len equ $ - cerrmsg
