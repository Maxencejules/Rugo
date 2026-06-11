; fsperm: permission proof. Runs as uid 100 against a path owned by
; root (the shell). Reports what the kernel allows: write-open, read-
; open, and unlink. With the default mode (owner rw, other r) the
; write and unlink must be denied; after a root chmod to 0xF they
; succeed. Entry: rdi = path (NUL terminated), rsi = len.

bits 64
default rel

section .text
global _start
_start:
    test rsi, rsi
    jz   .exit
    mov  r12, rdi            ; path

    ; write-open: open(path, O_WRONLY=1, 0)
    mov  rdi, r12
    mov  esi, 1
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .wdenied
    mov  rdi, rax
    mov  eax, 21             ; close
    int  0x80
    lea  rdi, [rel wok]
    mov  esi, wok_len
    jmp  .wprint
.wdenied:
    lea  rdi, [rel wno]
    mov  esi, wno_len
.wprint:
    xor  eax, eax
    int  0x80

    ; read-open: open(path, O_RDONLY=0, 0)
    mov  rdi, r12
    xor  esi, esi
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .rdenied
    mov  rdi, rax
    mov  eax, 21
    int  0x80
    lea  rdi, [rel rok]
    mov  esi, rok_len
    jmp  .rprint
.rdenied:
    lea  rdi, [rel rno]
    mov  esi, rno_len
.rprint:
    xor  eax, eax
    int  0x80

    ; unlink: fs_ctl(2, path, 0)
    mov  edi, 2
    mov  rsi, r12
    xor  edx, edx
    mov  eax, 47
    int  0x80
    cmp  rax, -1
    je   .udenied
    lea  rdi, [rel uok]
    mov  esi, uok_len
    jmp  .uprint
.udenied:
    lea  rdi, [rel uno]
    mov  esi, uno_len
.uprint:
    xor  eax, eax
    int  0x80

.exit:
    mov  eax, 2              ; sys_thread_exit
    int  0x80
.hang:
    jmp  .hang

section .rodata
wok: db "FSPERM: write ok", 10
wok_len equ $ - wok
wno: db "FSPERM: write denied", 10
wno_len equ $ - wno
rok: db "FSPERM: read ok", 10
rok_len equ $ - rok
rno: db "FSPERM: read denied", 10
rno_len equ $ - rno
uok: db "FSPERM: unlink ok", 10
uok_len equ $ - uok
uno: db "FSPERM: unlink denied", 10
uno_len equ $ - uno
