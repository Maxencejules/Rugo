; vmprobe: mmap / brk / munmap proof (sys_vm_ctl, id 50).
;
; Default mode: query and grow the program break, write/read brk memory,
; mmap an anonymous RW page, write/read it, munmap it, report ok.
;
; "ro" mode: mmap a PROT_READ page, read it (fine), then WRITE it - which
; must fault and be killed by the kernel (no CoW promotion: the page has no
; PTE_COW bit). The test asserts the kill and the absence of a "wrote"
; marker, proving prot enforcement.
;
; sys_vm_ctl: rdi=op, rsi=a2, rdx=a3, r10=a4.
;   op 1 mmap(va, sz, prot) -> va ; op 2 munmap(va, sz) -> 0 ;
;   op 3 brk(new) -> old break (new=0 queries).

bits 64
default rel

section .text
global _start
_start:
    cmp  rsi, 2
    jne  .normal
    mov  al, [rdi]
    cmp  al, 'r'
    je   .maybe_ro
    cmp  al, 'm'
    je   .maybe_mp
    jmp  .normal
.maybe_ro:
    mov  al, [rdi + 1]
    cmp  al, 'o'
    je   .romode
    jmp  .normal
.maybe_mp:
    mov  al, [rdi + 1]
    cmp  al, 'p'
    je   .mpmode
    jmp  .normal

.normal:
    ; brk(0) -> base
    mov  edi, 3
    xor  esi, esi
    mov  eax, 50
    int  0x80
    mov  r14, rax
    ; brk(base + 0x2000) -> old (== base)
    lea  rsi, [r14 + 0x2000]
    mov  edi, 3
    mov  eax, 50
    int  0x80
    cmp  rax, r14
    jne  .fail
    ; brk memory is usable
    mov  byte [r14], 0xAB
    mov  byte [r14 + 0x1500], 0xCD
    cmp  byte [r14], 0xAB
    jne  .fail
    cmp  byte [r14 + 0x1500], 0xCD
    jne  .fail
    ; mmap a RW page at the mmap base
    mov  edi, 1
    mov  rsi, 0x1200000
    mov  edx, 0x1000
    mov  r10d, 3
    mov  eax, 50
    int  0x80
    mov  rbx, 0x1200000
    cmp  rax, rbx
    jne  .fail
    mov  byte [rbx], 0x5A
    cmp  byte [rbx], 0x5A
    jne  .fail
    ; munmap it
    mov  edi, 2
    mov  rsi, 0x1200000
    mov  edx, 0x1000
    mov  eax, 50
    int  0x80
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

.romode:
    mov  edi, 1
    mov  rsi, 0x1220000
    mov  edx, 0x1000
    mov  r10d, 1                ; PROT_READ only
    mov  eax, 50
    int  0x80
    mov  rbx, 0x1220000
    cmp  rax, rbx
    jne  .fail
    lea  rdi, [rel romsg]
    mov  esi, romsg_len
    xor  eax, eax
    int  0x80
    mov  al, [rbx]             ; read is allowed
    mov  byte [rbx], 0x99      ; write must fault -> killed here
    ; only reached if prot was NOT enforced
    lea  rdi, [rel rowrotemsg]
    mov  esi, rowrotemsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

.mpmode:
    ; mmap a RW page, use it, then mprotect it read-only
    mov  edi, 1
    mov  rsi, 0x1230000
    mov  edx, 0x1000
    mov  r10d, 3
    mov  eax, 50
    int  0x80
    mov  rbx, 0x1230000
    cmp  rax, rbx
    jne  .fail
    mov  byte [rbx], 0x11       ; write while RW: fine
    ; mprotect(va, 0x1000, PROT_READ)
    mov  edi, 4
    mov  rsi, 0x1230000
    mov  edx, 0x1000
    mov  r10d, 1
    mov  eax, 50
    int  0x80
    cmp  rax, 0
    jne  .fail
    lea  rdi, [rel mpmsg]
    mov  esi, mpmsg_len
    xor  eax, eax
    int  0x80
    mov  al, [rbx]             ; read still allowed
    mov  byte [rbx], 0x22      ; write must now fault -> killed
    lea  rdi, [rel mpwrote]
    mov  esi, mpwrote_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
okmsg:       db "VMPROBE: ok", 10
okmsg_len    equ $ - okmsg
failmsg:     db "VMPROBE: FAIL", 10
failmsg_len  equ $ - failmsg
romsg:       db "VMPROBE: ro mapped", 10
romsg_len    equ $ - romsg
rowrotemsg:  db "VMPROBE: ro WROTE", 10
rowrotemsg_len equ $ - rowrotemsg
mpmsg:       db "VMPROBE: mp protected", 10
mpmsg_len    equ $ - mpmsg
mpwrote:     db "VMPROBE: mp WROTE", 10
mpwrote_len  equ $ - mpwrote
