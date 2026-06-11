; wc: count bytes from stdin (a pipe fd handed over by sys_spawn).
; Entry: rdi = args, rsi = len, rdx = stdin fd (-1 = none), rcx = stdout fd.
; Empty-pipe reads return -1 while a writer exists; 0 is EOF.

bits 64
default rel

section .text
global _start
_start:
    cmp  rdx, -1
    je   .err
    mov  r12, rdx            ; stdin fd
    xor  r13, r13            ; byte count
.read:
    mov  rdi, r12
    lea  rsi, [rel buf]
    mov  edx, 192
    mov  eax, 19             ; sys_read
    int  0x80
    cmp  rax, -1
    je   .retry
    test rax, rax
    jz   .done               ; EOF
    add  r13, rax
    jmp  .read
.retry:
    mov  eax, 3              ; sys_yield
    int  0x80
    jmp  .read
.done:
    ; print "WC: 0x" + two hex digits + " bytes\n"
    lea  rdi, [rel pfx]
    mov  esi, pfx_len
    xor  eax, eax
    int  0x80
    mov  rax, r13
    shr  rax, 4
    and  rax, 0x0F
    call put_hex_digit
    mov  rax, r13
    and  rax, 0x0F
    call put_hex_digit
    lea  rdi, [rel sfx]
    mov  esi, sfx_len
    xor  eax, eax
    int  0x80
    mov  rdi, r12
    mov  eax, 21             ; sys_close
    int  0x80
    mov  eax, 2              ; sys_thread_exit
    int  0x80
.err:
    lea  rdi, [rel emsg]
    mov  esi, emsg_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.hang:
    jmp  .hang

put_hex_digit:
    lea  rbx, [rel hexd]
    mov  al, [rbx + rax]
    mov  [rel digit], al
    lea  rdi, [rel digit]
    mov  esi, 1
    xor  eax, eax
    int  0x80
    ret

section .rodata
pfx: db "WC: 0x"
pfx_len equ $ - pfx
sfx: db " bytes", 10
sfx_len equ $ - sfx
hexd: db "0123456789ABCDEF"
emsg: db "wc: error", 10
emsg_len equ $ - emsg

section .bss
buf:   resb 192
digit: resb 1
