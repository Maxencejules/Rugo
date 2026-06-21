; pqsigprobe: public-key (Lamport) signature verify proof, from ring 3.
;
; sys_sigverify (id 63) runs the kernel's asymmetric Lamport verifier against its
; embedded public key + reference signature:
;   op 1 = the genuine signature        -> 1 (accept)
;   op 2 = a tampered message           -> 0 (reject)
;   op 3 = a tampered signature         -> 0 (reject)
; The kernel holds only the PUBLIC key, so it can verify but never forge. The
; probe fails unless the genuine signature is accepted and BOTH forgeries are
; rejected. sys_sigverify: rdi=op (id 63) -> 1/0.

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 1                ; genuine signature
    mov  eax, 63
    int  0x80
    cmp  rax, 1
    jne  .fail                 ; must ACCEPT

    mov  edi, 2                ; tampered message
    mov  eax, 63
    int  0x80
    cmp  rax, 0
    jne  .fail                 ; must REJECT

    mov  edi, 3                ; tampered signature
    mov  eax, 63
    int  0x80
    cmp  rax, 0
    jne  .fail                 ; must REJECT

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
okmsg:   db "PQSIGAPP: verify ok forge rejected", 10
okmsg_len equ $ - okmsg
failmsg: db "PQSIGAPP: FAIL", 10
failmsg_len equ $ - failmsg
