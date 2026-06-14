; libdl: a real ELF shared object for the kernel dynamic linker (full-os V.11).
; Built with nasm -f elf64 + rust-lld -shared. Exports:
;   addtwo(x) = x + 2          (position-independent; tests symbol resolve+call)
;   getval()  = 42             (via a pointer word that needs an R_X86_64_RELATIVE
;                               relocation applied at load -> tests relocation)
bits 64
default rel

section .text
global addtwo:function
addtwo:
    lea  rax, [rdi + 2]
    ret

global getval:function
getval:
    mov  rax, [rel pmyval]   ; load the (relocated) pointer to myval
    mov  rax, [rax]          ; deref -> 42
    ret

section .data
myval:  dq 42
global pmyval:data
pmyval: dq myval             ; absolute addr of myval -> R_X86_64_RELATIVE
