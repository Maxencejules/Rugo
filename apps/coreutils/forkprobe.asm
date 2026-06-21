; forkprobe: copy-on-write fork proof.
;
; The app writes a sentinel into a global, forks (sys_proc_ctl op 1), then:
;   * the child writes a DIFFERENT value to that same global (its private
;     CoW copy) and confirms it reads back its own value, then exits;
;   * the parent yields so the child runs, then confirms its OWN copy of
;     the global is UNCHANGED - which only holds if fork gave the child a
;     copy-on-write duplicate, not a shared page.
;
; fork returns the child tid in the parent (rax != 0) and 0 in the child.

bits 64
default rel

section .text
global _start
_start:
    ; establish the page contents before forking
    mov  rax, 0x1111111111111111
    mov  [rel shared], rax

    ; fork: sys_proc_ctl(op=1)
    mov  edi, 1
    mov  eax, 51
    int  0x80
    test rax, rax
    jz   .child

; ---- parent ----
    ; let the child run (and break CoW) a few times
    mov  r12, 6
.pyield:
    mov  eax, 3                 ; sys_yield
    int  0x80
    dec  r12
    jnz  .pyield
    ; our copy must be untouched by the child's write
    mov  rax, 0x1111111111111111
    cmp  [rel shared], rax
    jne  .pfail
    lea  rdi, [rel pmsg]
    mov  esi, pmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2                 ; sys_thread_exit
    int  0x80
.pfail:
    lea  rdi, [rel pfailmsg]
    mov  esi, pfailmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

; ---- child ----
.child:
    mov  rax, 0x2222222222222222
    mov  [rel shared], rax       ; write -> CoW break, private copy
    cmp  [rel shared], rax       ; we must see our own value
    jne  .cfail
    lea  rdi, [rel cmsg]
    mov  esi, cmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.cfail:
    lea  rdi, [rel cfailmsg]
    mov  esi, cfailmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
shared:   dq 0
pmsg:     db "FORKPROBE: parent ok cow-isolated", 10
pmsg_len  equ $ - pmsg
pfailmsg: db "FORKPROBE: parent FAIL clobbered", 10
pfailmsg_len equ $ - pfailmsg
cmsg:     db "FORKPROBE: child ok wrote private", 10
cmsg_len  equ $ - cmsg
cfailmsg: db "FORKPROBE: child FAIL", 10
cfailmsg_len equ $ - cfailmsg
