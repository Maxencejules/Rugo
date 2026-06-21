; ptyprobe: pty pair proof (full-os guide Part V.11 TTY/pty).
;
; openpty via ioctl(op=2, id 56) -> rax = (slave_fd << 32) | master_fd.
; Bytes written to the master are readable from the slave and vice versa.
; open=18 read=19 write=20 ioctl=56; console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    ; openpty
    mov  edi, 2
    mov  eax, 56
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax
    mov  r13d, r12d             ; master = low 32 bits
    mov  r14, r12
    shr  r14, 32                ; slave  = high 32 bits

    ; master -> slave: write "ptyhello" to master
    mov  rdi, r13
    lea  rsi, [rel msg1]
    mov  edx, msg1_len
    mov  eax, 20
    int  0x80
    cmp  rax, msg1_len
    jne  .fail
    ; read it back from the slave
    mov  rdi, r14
    lea  rsi, [rel buf]
    mov  edx, 64
    mov  eax, 19
    int  0x80
    cmp  rax, msg1_len
    jne  .fail
    lea  rdi, [rel buf]
    mov  esi, msg1_len
    xor  eax, eax
    int  0x80

    ; slave -> master: write "ptyback!" to slave
    mov  rdi, r14
    lea  rsi, [rel msg2]
    mov  edx, msg2_len
    mov  eax, 20
    int  0x80
    cmp  rax, msg2_len
    jne  .fail
    ; read it back from the master
    mov  rdi, r13
    lea  rsi, [rel buf]
    mov  edx, 64
    mov  eax, 19
    int  0x80
    cmp  rax, msg2_len
    jne  .fail
    lea  rdi, [rel buf]
    mov  esi, msg2_len
    xor  eax, eax
    int  0x80

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
msg1:    db "ptyhello"
msg1_len equ $ - msg1
msg2:    db "ptyback!"
msg2_len equ $ - msg2
okmsg:   db "PTYPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "PTYPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
buf: resb 64
