; epollprobe: epoll readiness proof (sys_epoll, ABI v3.x id 55).
;
; Creates an epoll instance and a pipe, registers the pipe read end for
; EPOLLIN, and proves LEVEL-TRIGGERED readiness: while the pipe is empty
; epoll_wait reports 0 ready; after one byte is written it reports 1 ready,
; with the returned fd == the read end and EPOLLIN (0x1) set in revents.
;
; sys_epoll: rdi=op, rsi=arg2, rdx=arg3, r10=arg4, eax=55.
;   op 1 create -> rax=epfd; op 2 ctl_add(ep,fd,events); op 3 wait(ep,buf,max).
; pipe via sys_fs_ctl id 47 op 4 -> rax = rfd<<8 | wfd.

bits 64
default rel

section .text
global _start
_start:
    ; epoll_create -> r12 = epfd
    mov  edi, 1
    mov  eax, 55
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r12, rax

    ; pipe() -> r13 = rfd, r14 = wfd
    mov  edi, 4              ; FS_CTL_PIPE
    xor  esi, esi
    xor  edx, edx
    mov  eax, 47
    int  0x80
    cmp  rax, -1
    je   .fail
    mov  r13, rax
    shr  r13, 8             ; rfd = pair >> 8
    mov  r14, rax
    and  r14, 0xFF          ; wfd = pair & 0xFF

    ; epoll_ctl_add(epfd, rfd, EPOLLIN=1)
    mov  edi, 2
    mov  rsi, r12
    mov  rdx, r13
    mov  r10, 1
    mov  eax, 55
    int  0x80
    cmp  rax, -1
    je   .fail

    ; epoll_wait #1: pipe empty -> 0 ready
    mov  edi, 3
    mov  rsi, r12
    lea  rdx, [rel evbuf]
    mov  r10, 4
    mov  eax, 55
    int  0x80
    test rax, rax
    jnz  .fail              ; must be 0

    ; write one byte into the pipe write end
    mov  rdi, r14
    lea  rsi, [rel xbyte]
    mov  edx, 1
    mov  eax, 20            ; sys_write
    int  0x80
    cmp  rax, 1
    jne  .fail

    ; epoll_wait #2: now readable -> 1 ready
    mov  edi, 3
    mov  rsi, r12
    lea  rdx, [rel evbuf]
    mov  r10, 4
    mov  eax, 55
    int  0x80
    cmp  rax, 1
    jne  .fail

    ; the ready record's fd must equal rfd and revents must have EPOLLIN
    mov  eax, [rel evbuf]        ; fd (i32, zero-extended)
    cmp  rax, r13
    jne  .fail
    movzx eax, word [rel evbuf + 4]  ; revents (u16)
    and  eax, 1
    jz   .fail

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

section .rodata
okmsg:   db "EPOLLPROBE: ready ok", 10
okmsg_len equ $ - okmsg
failmsg: db "EPOLLPROBE: FAIL", 10
failmsg_len equ $ - failmsg
xbyte:   db "x"

section .bss
evbuf:   resb 32
