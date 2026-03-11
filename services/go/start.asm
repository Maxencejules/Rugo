; Rugo x86_64 user-space startup + syscall wrappers + runtime stubs.
; Built as TinyGo bare-metal glue for the canonical Go userspace lane.

bits 64
default rel

section .text._start
global _start
extern main
_start:
    call main
    jmp main.haltForever

section .text
extern goSpawnedThreadMain

global main.sysDebugWrite
main.sysDebugWrite:
    xor  eax, eax
    int  0x80
    ret

global main.sysThreadSpawn
main.sysThreadSpawn:
    mov  eax, 1
    int  0x80
    ret

global main.sysThreadExit
main.sysThreadExit:
    mov  eax, 2
    int  0x80
    ret

global main.sysYield
main.sysYield:
    mov  eax, 3
    int  0x80
    ret

global main.sysIpcSend
main.sysIpcSend:
    mov  eax, 8
    int  0x80
    ret

global main.sysIpcRecv
main.sysIpcRecv:
    mov  eax, 9
    int  0x80
    ret

global main.sysTimeNow
main.sysTimeNow:
    mov  eax, 10
    int  0x80
    ret

global main.sysSvcRegister
main.sysSvcRegister:
    mov  eax, 11
    int  0x80
    ret

global main.sysSvcLookup
main.sysSvcLookup:
    mov  eax, 12
    int  0x80
    ret

global main.sysIpcEndpointCreate
main.sysIpcEndpointCreate:
    mov  eax, 17
    int  0x80
    ret

global main.sysSpawnEntry
main.sysSpawnEntry:
    lea  rax, [rel go_spawn_entry]
    ret

go_spawn_entry:
    call goSpawnedThreadMain
    mov  eax, 2
    int  0x80
    jmp main.haltForever

global main.haltForever
main.haltForever:
    hlt
    jmp  main.haltForever

global runtime.alloc
runtime.alloc:
    mov  rax, qword [0x7FF000]
    test rax, rax
    jnz  .has_ptr
    mov  rax, 0x7FF008
.has_ptr:
    add  rdi, 7
    and  rdi, ~7
    lea  rdx, [rax + rdi]
    mov  qword [0x7FF000], rdx
    ret

global getrandom
getrandom:
    xor  eax, eax
    ret

global tinygo_register_fatal_signals
tinygo_register_fatal_signals:
    ret

global memset
memset:
    push rdi
    mov  rax, rsi
    mov  rcx, rdx
    rep  stosb
    pop  rax
    ret

global memcpy
memcpy:
    mov  rax, rdi
    mov  rcx, rdx
    rep  movsb
    ret

global memmove
memmove:
    mov  rax, rdi
    cmp  rdi, rsi
    jbe  .fwd
    lea  rsi, [rsi + rdx - 1]
    lea  rdi, [rdi + rdx - 1]
    std
    mov  rcx, rdx
    rep  movsb
    cld
    ret
.fwd:
    mov  rcx, rdx
    rep  movsb
    ret

global write
write:
    mov  rdi, rsi
    mov  rsi, rdx
    xor  eax, eax
    int  0x80
    ret

global abort
abort:
    jmp  main.haltForever

global exit
exit:
    jmp  main.haltForever

global _exit
_exit:
    jmp  main.haltForever
