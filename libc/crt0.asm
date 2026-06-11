; rlibc crt0: process entry for C programs run from the package store.
; Kernel hands: rdi = args pointer (NUL terminated), rsi = args length,
; rdx = stdin fd (-1 = none), rcx = stdout fd (-1 = console).
; Calls main(void); main's return value becomes the exit code.

bits 64
default rel

extern main
extern rugo_args
extern rugo_args_len
extern rugo_stdin_fd
extern rugo_stdout_fd

section .text
global _start
_start:
    mov  [rugo_args], rdi
    mov  [rugo_args_len], rsi
    mov  [rugo_stdin_fd], rdx
    mov  [rugo_stdout_fd], rcx
    call main
    mov  rdi, rax
    mov  eax, 2              ; sys_thread_exit
    int  0x80
.hang:
    jmp  .hang

; long rugo_syscall3(long nr, long a1, long a2, long a3)
; C side is compiled -mabi=sysv: rdi=nr rsi=a1 rdx=a2 rcx=a3.
global rugo_syscall3
rugo_syscall3:
    mov  rax, rdi            ; nr
    mov  rdi, rsi            ; a1
    mov  rsi, rdx            ; a2
    mov  rdx, rcx            ; a3
    int  0x80
    ret

; long rugo_syscall6(long nr, a1, a2, a3, a4, a5, a6)
; SysV: rdi rsi rdx rcx r8 r9 + stack(a6)
global rugo_syscall6
rugo_syscall6:
    mov  rax, rdi            ; nr
    mov  rdi, rsi            ; a1
    mov  rsi, rdx            ; a2
    mov  rdx, rcx            ; a3
    mov  r10, r8             ; a4
    mov  r8, r9              ; a5
    mov  r9, [rsp + 8]       ; a6
    int  0x80
    ret
