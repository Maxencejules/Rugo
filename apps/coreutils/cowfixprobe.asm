; cowfixprobe: regression guard for two fork-time memory-isolation bugs.
;
;  P1 (mprotect must break CoW): a forked CoW page made writable via sys_vm_ctl
;     op 4 must NOT keep aliasing the sibling's physical frame. Here the child
;     mprotect()s the shared page back to RW and stores S2; the parent (after
;     waitpid, so ordering is deterministic) must still read its own S1. A leak
;     means mprotect granted write on the still-shared frame -> isolation broken.
;  P2 (fork must inherit the program break): after the parent grows its break
;     with brk(), the child's brk(0) must report the INHERITED break, not reset
;     to the base (address_space_fork cloned the grown heap pages).
;
; ABI: sys_vm_ctl id 50 (rdi=op, rsi=a2, rdx=a3, r10=a4): op 1 mmap(va,sz,prot),
; op 3 brk(new) -> old, op 4 mprotect(va,sz,prot). fork = sys_proc_ctl(rdi=1),
; eax=51 -> child tid in parent / 0 in child. waitpid = eax=22 (rdi=pid,
; rsi=status, rdx=opts). console write = eax=0 (rdi=buf, rsi=len). exit = eax=2.
; All GP registers survive int 0x80 (a DPL=3 interrupt gate -- isr_common saves
; and restores them) except rax, and the child inherits the parent's registers
; across fork (r14=brk base, r15=mmap VA).

bits 64
default rel

section .text
global _start
_start:
    ; --- query + grow the program break (P2 setup, runs in the parent) ---
    mov  edi, 3                 ; brk(0) -> base
    xor  esi, esi
    mov  eax, 50
    int  0x80
    mov  r14, rax               ; r14 = brk base
    lea  rsi, [r14 + 0x2000]     ; brk(base + 0x2000): grow two pages
    mov  edi, 3
    mov  eax, 50
    int  0x80
    cmp  rax, r14               ; old break must equal base
    jne  .setupfail

    ; --- mmap a RW page and seed it with S1 (P1 setup) ---
    mov  r15, 0x1240000
    mov  edi, 1                 ; mmap(va, 0x1000, RW)
    mov  rsi, r15
    mov  edx, 0x1000
    mov  r10d, 3
    mov  eax, 50
    int  0x80
    cmp  rax, r15
    jne  .setupfail
    mov  rax, 0x1111111111111111
    mov  [r15], rax             ; S1

    ; --- fork ---
    mov  edi, 1
    mov  eax, 51
    int  0x80
    test rax, rax
    jz   .child

; ---- parent ----
    mov  rbx, rax               ; child tid
    mov  rdi, rbx               ; waitpid(child): block until it has run+written
    xor  esi, esi               ; status ptr = 0 (don't care)
    xor  edx, edx
    mov  eax, 22
    int  0x80
    cmp  rax, -1
    je   .waiterr
    mov  rax, 0x1111111111111111
    cmp  [r15], rax             ; our page must STILL hold S1, not the child's S2
    jne  .leak
    lea  rdi, [rel pokmsg]
    mov  esi, pokmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.leak:
    lea  rdi, [rel leakmsg]
    mov  esi, leakmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.waiterr:
    lea  rdi, [rel waitmsg]
    mov  esi, waitmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

; ---- child ----
.child:
    ; P2: brk(0) must return the INHERITED break (base + 0x2000), not the base.
    mov  edi, 3
    xor  esi, esi
    mov  eax, 50
    int  0x80
    lea  rcx, [r14 + 0x2000]
    cmp  rax, rcx
    jne  .cbrkfail
    lea  rdi, [rel cbrkmsg]
    mov  esi, cbrkmsg_len
    xor  eax, eax
    int  0x80
    jmp  .cmprot
.cbrkfail:
    lea  rdi, [rel cbrkfailmsg]
    mov  esi, cbrkfailmsg_len
    xor  eax, eax
    int  0x80
.cmprot:
    ; P1: make our (post-fork CoW) page writable and store S2. With the fix this
    ; takes a private copy; with the bug it writes through the shared frame.
    mov  edi, 4                 ; mprotect(va, 0x1000, RW)
    mov  rsi, r15
    mov  edx, 0x1000
    mov  r10d, 3
    mov  eax, 50
    int  0x80
    mov  rax, 0x2222222222222222
    mov  [r15], rax             ; S2 via the mprotect'd page
    lea  rdi, [rel cwrotemsg]
    mov  esi, cwrotemsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

.setupfail:
    lea  rdi, [rel setupmsg]
    mov  esi, setupmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
pokmsg:         db "COWFIX: parent mp-isolated ok", 10
pokmsg_len      equ $ - pokmsg
leakmsg:        db "COWFIX: parent mp-leak FAIL", 10
leakmsg_len     equ $ - leakmsg
waitmsg:        db "COWFIX: FAIL waiterr", 10
waitmsg_len     equ $ - waitmsg
cbrkmsg:        db "COWFIX: child brk-inherited ok", 10
cbrkmsg_len     equ $ - cbrkmsg
cbrkfailmsg:    db "COWFIX: child brk-reset FAIL", 10
cbrkfailmsg_len equ $ - cbrkfailmsg
cwrotemsg:      db "COWFIX: child mprotect-wrote", 10
cwrotemsg_len   equ $ - cwrotemsg
setupmsg:       db "COWFIX: FAIL setup", 10
setupmsg_len    equ $ - setupmsg
