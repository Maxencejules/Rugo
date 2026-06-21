; page3probe: diagnostic for the exec loader's handling of apps that span a
; THIRD page. A magic global is padded into the 3rd page (vaddr >= 0x1402000),
; file-backed (initialized). The probe reads it (must equal the magic -> proves
; as_copyout loaded the 3rd page), then writes+reads it back (3rd-page write).
; console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    mov  rax, [bigvar]
    mov  rbx, 0xCAFEBABEDEADBEEF
    cmp  rax, rbx
    jne  .fail
    mov  rcx, 0x1122334455667788
    mov  [bigvar], rcx
    mov  rdx, [bigvar]
    cmp  rdx, rcx
    jne  .fail
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

section .data
okmsg:   db "PAGE3: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "PAGE3: FAIL", 10
failmsg_len equ $ - failmsg
times 0x2200 db 0          ; pad bigvar into the 3rd page (offset > 0x2000)
bigvar:  dq 0xCAFEBABEDEADBEEF
