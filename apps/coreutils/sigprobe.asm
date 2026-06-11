; sigprobe: signal proof. Default mode registers a handler, sends
; itself signal 15, and proves delivery (handler runs with rdi=15) and
; sigreturn (execution resumes after the kill syscall and exits
; cleanly). With argument "die" it registers nothing and self-kills:
; the kernel's default action must terminate the task.

bits 64
default rel

section .text
global _start
_start:
    ; args: rdi = ptr, rsi = len. "die" selects the no-handler mode.
    cmp  rsi, 3
    jne  .handled_mode
    cmp  byte [rdi], 'd'
    jne  .handled_mode
    cmp  byte [rdi+1], 'i'
    jne  .handled_mode
    cmp  byte [rdi+2], 'e'
    je   .die_mode

.handled_mode:
    ; signal_ctl(1, handler, 0)
    mov  edi, 1
    lea  rsi, [rel handler]
    xor  edx, edx
    mov  eax, 48
    int  0x80
    ; signal_ctl(2, SELF, 15) - delivered before this returns
    mov  edi, 2
    mov  rsi, -1
    mov  edx, 15
    mov  eax, 48
    int  0x80
    ; we resume here via sigreturn
    cmp  byte [rel flag], 1
    jne  .bad
    lea  rdi, [rel resumed]
    mov  esi, resumed_len
    xor  eax, eax
    int  0x80
    mov  eax, 2              ; sys_thread_exit
    int  0x80

.die_mode:
    ; no handler registered: default action must kill us mid-syscall
    mov  edi, 2
    mov  rsi, -1
    mov  edx, 15
    mov  eax, 48
    int  0x80
    ; unreachable
.bad:
    lea  rdi, [rel bad]
    mov  esi, bad_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80
.hang:
    jmp  .hang

handler:
    ; rdi = signal number
    cmp  rdi, 15
    jne  .wrong
    lea  rdi, [rel hmsg]
    mov  esi, hmsg_len
    xor  eax, eax
    int  0x80
    mov  byte [rel flag], 1
    mov  edi, 3              ; sigreturn
    mov  eax, 48
    int  0x80
.wrong:
    lea  rdi, [rel bad]
    mov  esi, bad_len
    xor  eax, eax
    int  0x80
    mov  eax, 2
    int  0x80

section .rodata
hmsg: db "SIGPROBE: handler sig=15", 10
hmsg_len equ $ - hmsg
resumed: db "SIGPROBE: resumed after handler", 10
resumed_len equ $ - resumed
bad: db "SIGPROBE: bad path", 10
bad_len equ $ - bad

section .bss
flag: resb 1
