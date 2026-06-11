; cat: print a file from the /data tree. Entry: rdi = path (NUL
; terminated by the kernel), rsi = len, rdx = stdin fd (unused),
; rcx = stdout fd (-1 = console via debug_write; otherwise the file
; content is written to that fd, e.g. a pipe write end).

bits 64
default rel

section .text
global _start
_start:
    mov  [rel out_fd], rcx
    test rsi, rsi
    jz   .err
    ; open(path, RDONLY=0, 0)
    xor  esi, esi
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .err
    mov  r12, rax            ; fd
.loop:
    mov  rdi, r12
    lea  rsi, [rel buf]
    mov  edx, 192
    mov  eax, 19             ; sys_read
    int  0x80
    cmp  rax, -1
    je   .close_err
    test rax, rax
    jz   .done
    mov  r13, rax            ; bytes
    mov  rcx, [rel out_fd]
    cmp  rcx, -1
    jne  .to_fd
    ; debug_write(buf, n)
    lea  rdi, [rel buf]
    mov  rsi, r13
    xor  eax, eax
    int  0x80
    jmp  .loop
.to_fd:
    ; sys_write(out_fd, buf, n); retry while the pipe is full
    mov  rdi, rcx
    lea  rsi, [rel buf]
    mov  rdx, r13
    mov  eax, 20             ; sys_write
    int  0x80
    cmp  rax, -1
    jne  .loop
    mov  eax, 3              ; sys_yield, pipe full
    int  0x80
    jmp  .to_fd
.done:
    mov  rdi, r12
    mov  eax, 21             ; sys_close
    int  0x80
    mov  rcx, [rel out_fd]
    cmp  rcx, -1
    je   .exit
    mov  rdi, rcx            ; close the pipe end: EOF for the reader
    mov  eax, 21
    int  0x80
.exit:
    mov  eax, 2
    int  0x80
.close_err:
    mov  rdi, r12
    mov  eax, 21
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

section .rodata
emsg: db "cat: error", 10
emsg_len equ $ - emsg

section .bss
buf:    resb 192
out_fd: resq 1
