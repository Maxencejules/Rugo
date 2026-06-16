; libdl2: a SECOND translation unit for the dynamic-linker test. Defining gvar +
; extadd here (not in libdl.asm) makes libdl.asm's references to them CROSS-OBJECT
; (external), so nasm emits clean GOTPCREL/PLT relocs that rust-lld -shared then
; resolves to R_X86_64_GLOB_DAT / R_X86_64_JUMP_SLOT in the final .so (a
; same-object reference would be bound directly with no symbolic relocation).
bits 64
default rel

section .text
global extadd:function
extadd:
    lea  rax, [rdi + 2]   ; extadd(x) = x + 2
    ret

section .data
global gvar:data
gvar:   dq 99
