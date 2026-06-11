; ls: list a /data directory. Entry: rdi = path (NUL terminated), rsi = len.
; Directory reads return packed 32-byte dirents: name[24] kind u8 pad[3]
; size u32 (kind 2 = directory).

bits 64
default rel

section .text
global _start
_start:
    test rsi, rsi
    jz   .err
    xor  esi, esi            ; RDONLY
    xor  edx, edx
    mov  eax, 18             ; sys_open
    int  0x80
    cmp  rax, -1
    je   .err
    mov  r12, rax            ; fd
.read:
    mov  rdi, r12
    lea  rsi, [rel recs]
    mov  edx, 128
    mov  eax, 19             ; sys_read
    int  0x80
    cmp  rax, -1
    je   .done
    test rax, rax
    jz   .done
    mov  r13, rax            ; bytes
    xor  r14, r14            ; offset
.entry:
    lea  rbx, [rel recs]
    add  rbx, r14
    ; name length (up to 24, stop at NUL)
    xor  rcx, rcx
.scan:
    cmp  rcx, 24
    jae  .have_len
    cmp  byte [rbx + rcx], 0
    je   .have_len
    inc  rcx
    jmp  .scan
.have_len:
    mov  rdi, rbx
    mov  rsi, rcx
    xor  eax, eax            ; debug_write(name, len)
    int  0x80
    cmp  byte [rbx + 24], 2
    jne  .nl
    lea  rdi, [rel slash]
    mov  esi, 1
    xor  eax, eax
    int  0x80
.nl:
    lea  rdi, [rel nl]
    mov  esi, 1
    xor  eax, eax
    int  0x80
    add  r14, 32
    cmp  r14, r13
    jb   .entry
    jmp  .read
.done:
    mov  rdi, r12
    mov  eax, 21             ; sys_close
    int  0x80
    mov  eax, 2
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
slash: db "/"
nl:    db 10
emsg:  db "ls: error", 10
emsg_len equ $ - emsg

section .bss
recs: resb 128
