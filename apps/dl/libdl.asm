; libdl: a real ELF shared object for the kernel dynamic linker (full-os V.11).
; Built with nasm -f elf64 + rust-lld -shared (libdl.asm + libdl2.asm). Exports:
;   addtwo(x) = x + 2          (position-independent; tests symbol resolve+call)
;   getval()  = 42             (via a pointer word that needs an R_X86_64_RELATIVE
;                               relocation applied at load -> tests relocation)
;   getgvar() = 99             (reads gvar [defined in libdl2] via the GOT -> tests
;                               R_X86_64_GLOB_DAT: the GOT slot must be relocated)
;   callsum() = extadd(40)=42  (calls extadd [defined in libdl2] via the PLT -> tests
;                               R_X86_64_JUMP_SLOT: the .got.plt slot must be relocated)
bits 64
default rel

extern gvar
extern extadd

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

global getgvar:function
getgvar:
    mov  rax, [rel gvar wrt ..got]   ; cross-object GOT slot -> &gvar (GLOB_DAT)
    mov  rax, [rax]                  ; deref -> 99
    ret

global callsum:function
callsum:
    mov  edi, 40
    call extadd wrt ..plt    ; cross-object PLT call -> extadd (JUMP_SLOT) -> 42
    ret

section .data
myval:  dq 42
global pmyval:data
pmyval: dq myval             ; absolute addr of myval -> R_X86_64_RELATIVE
