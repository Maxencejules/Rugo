; lseekprobe: lseek proof (sys_fs_ctl op 6), all three whences.
;
; Creates /data/lstst with "ABCDE", reopens it, then: SEEK_SET 2 -> read "CDE";
; SEEK_CUR -3 (from offset 5) -> read "C"; SEEK_END -2 (filesize 5) -> read "DE".
; open=18, read=19, write=20, close=21, fs_ctl=47 (op 6 = lseek: rsi = fd | whence<<32
; [0=SET,1=CUR,2=END], rdx = offset/delta). RDONLY=0, WRONLY|CREATE=5.

bits 64
default rel

section .text
global _start
_start:
    ; create + write
    lea  rdi, [rel path]
    mov  esi, 5
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax
    mov  rdi, r12
    lea  rsi, [rel data]
    mov  edx, 5
    mov  eax, 20
    int  0x80
    cmp  rax, 5
    jne  .fail
    mov  rdi, r12
    mov  eax, 21
    int  0x80
    ; reopen read-only
    lea  rdi, [rel path]
    xor  esi, esi
    xor  edx, edx
    mov  eax, 18
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax
    ; lseek(fd, 2)
    mov  edi, 6
    mov  rsi, r12
    mov  edx, 2
    mov  eax, 47
    int  0x80
    cmp  rax, 2
    jne  .fail
    ; read 3 bytes -> "CDE"
    mov  rdi, r12
    lea  rsi, [rel rbuf]
    mov  edx, 3
    mov  eax, 19
    int  0x80
    cmp  rax, 3
    jne  .fail
    cmp  byte [rel rbuf], 'C'
    jne  .fail
    cmp  byte [rel rbuf + 1], 'D'
    jne  .fail
    cmp  byte [rel rbuf + 2], 'E'
    jne  .fail
    ; SEEK_CUR -3 (whence=1): from current offset 5 -> 2
    mov  edi, 6
    mov  rsi, r12
    bts  rsi, 32
    mov  rdx, -3
    mov  eax, 47
    int  0x80
    cmp  rax, 2
    jne  .fail
    ; read 1 -> 'C'
    mov  rdi, r12
    lea  rsi, [rel rbuf]
    mov  edx, 1
    mov  eax, 19
    int  0x80
    cmp  rax, 1
    jne  .fail
    cmp  byte [rel rbuf], 'C'
    jne  .fail
    ; SEEK_END -2 (whence=2): filesize 5 -> 3
    mov  edi, 6
    mov  rsi, r12
    bts  rsi, 33
    mov  rdx, -2
    mov  eax, 47
    int  0x80
    cmp  rax, 3
    jne  .fail
    ; read 2 -> "DE"
    mov  rdi, r12
    lea  rsi, [rel rbuf]
    mov  edx, 2
    mov  eax, 19
    int  0x80
    cmp  rax, 2
    jne  .fail
    cmp  byte [rel rbuf], 'D'
    jne  .fail
    cmp  byte [rel rbuf + 1], 'E'
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
path:    db "/data/lstst", 0
data:    db "ABCDE"
okmsg:   db "LSEEKPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "LSEEKPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
rbuf: resb 3
