; ps: enumerate live kernel tasks via sys_proc_info (id 28). Prints one
; "PS: tid 0xNN" line per task the kernel reports.

bits 64
default rel

section .text
global _start
_start:
    xor  r12, r12            ; tid
.next:
    cmp  r12, 32
    jae  .done
    mov  rdi, r12
    lea  rsi, [rel info]
    mov  edx, 136            ; R4_PROC_INFO_EXT_SIZE
    mov  eax, 28             ; sys_proc_info
    int  0x80
    cmp  rax, -1
    je   .skip
    ; print "PS: tid 0x" + two hex digits + "\n"
    lea  rdi, [rel pfx]
    mov  esi, pfx_len
    xor  eax, eax
    int  0x80
    mov  rax, r12
    shr  rax, 4
    call put_hex_digit
    mov  rax, r12
    and  rax, 0x0F
    call put_hex_digit
    lea  rdi, [rel nl]
    mov  esi, 1
    xor  eax, eax
    int  0x80
.skip:
    inc  r12
    jmp  .next
.done:
    mov  eax, 2              ; sys_thread_exit
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
pfx: db "PS: tid 0x"
pfx_len equ $ - pfx
hexd: db "0123456789ABCDEF"
nl: db 10

section .bss
info:  resb 136
digit: resb 1
