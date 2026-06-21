; waitprobe: regression guard for cross-address-space waitpid status delivery.
;
; A spawned app (private address space) forks a child and waitpid()s for it
; with a status pointer. The kernel wakes the parent from the CHILD's (or
; SHARED) address space, so it must write the exit status into the PARENT's
; table, not the current one. We pre-poison the status word; after wait it
; must read back 0 (the child's exit status) - proof it was delivered into
; our own address space.
;
; fork = sys_proc_ctl(rdi=1), eax=51. waitpid = sys_wait(rdi=pid,
; rsi=status_ptr, rdx=opts), eax=22.

bits 64
default rel

section .text
global _start
_start:
    mov  rax, 0xDEADBEEFDEADBEEF
    mov  [rel statusvar], rax
    ; fork
    mov  edi, 1
    mov  eax, 51
    int  0x80
    test rax, rax
    jz   .child
    ; parent: rbx = child tid
    mov  rbx, rax
    mov  rdi, rbx
    lea  rsi, [rel statusvar]
    xor  edx, edx
    mov  eax, 22
    int  0x80
    cmp  rax, -1
    je   .waiterr
    ; the status word must now be the child's exit status (0), written into
    ; OUR address space - not left poisoned, not written elsewhere.
    mov  rax, [rel statusvar]
    test rax, rax
    jnz  .statusbad
    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.child:
    mov  eax, 2
    int  0x80
.waiterr:
    lea  rdi, [rel waiterrmsg]
    mov  esi, waiterrmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.statusbad:
    lea  rdi, [rel statusbadmsg]
    mov  esi, statusbadmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
statusvar: dq 0
okmsg:   db "WAITPROBE: status ok", 10
okmsg_len equ $ - okmsg
waiterrmsg: db "WAITPROBE: FAIL wait err", 10
waiterrmsg_len equ $ - waiterrmsg
statusbadmsg: db "WAITPROBE: FAIL status bad", 10
statusbadmsg_len equ $ - statusbadmsg
