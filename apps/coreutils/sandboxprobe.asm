; sandboxprobe: syscall-allowlist proof (sys_sandbox, id 59).
;
; Narrows its own allowlist to syscall 0 only (debug_write; exit stays
; force-allowed), then attempts sys_yield (3), which the kernel must deny
; (-1). Reports the denial and exits. Proves per-task pledge-style
; sandboxing with monotonic narrowing.

bits 64
default rel

section .text
global _start
_start:
    ; sandbox(allow = 1<<0)
    mov  rdi, 1
    mov  eax, 59
    int  0x80
    cmp  rax, 0
    jne  .fail
    ; attempt a now-forbidden syscall (yield)
    mov  eax, 3
    int  0x80
    cmp  rax, -1
    jne  .fail
    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax              ; debug_write - still allowed
    int  0x80
    mov  eax, 2               ; exit - still allowed
    int  0x80
.fail:
    lea  rdi, [rel failmsg]
    mov  esi, failmsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .data
okmsg:   db "SANDBOXPROBE: denied ok", 10
okmsg_len equ $ - okmsg
failmsg: db "SANDBOXPROBE: FAIL", 10
failmsg_len equ $ - failmsg
