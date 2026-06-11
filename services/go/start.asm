; Rugo x86_64 user-space startup + syscall wrappers + runtime stubs.
; Built as TinyGo bare-metal glue for the canonical Go userspace lane.

bits 64
default rel

; TinyGo bump heap lives in the kernel's demand-paged window
; [0x100_0000, 0x180_0000). Pages are zero-filled on first touch by the
; page-fault handler, so allocations come back zeroed without an explicit
; memset. The old fixed heap at 0x7F4000 shared its 16 KiB with the
; spawned tasks' stacks (stack tops at 0x7F8000 and up) and overflowed
; into them once per-line log buffers landed on the heap.
; 0x110_0000 leaves the first 1 MiB of the window to the boot-time
; demand probe.
%define GO_HEAP_PTR  0x1100000
%define GO_HEAP_BASE 0x1100008

section .text._start
global _start
extern main
_start:
    ; Seed the bump-heap pointer while still single-threaded; the first
    ; write demand-maps the page.
    mov  rax, GO_HEAP_BASE
    mov  qword [abs GO_HEAP_PTR], rax
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

global main.sysWait
main.sysWait:
    mov  eax, 22
    int  0x80
    ret

global main.sysSpawn
main.sysSpawn:
    mov  r10, rcx            ; 4th SysV arg -> 4th syscall arg
    mov  eax, 46
    int  0x80
    ret

global main.sysFsCtl
main.sysFsCtl:
    mov  eax, 47
    int  0x80
    ret

global main.sysNetQuery
main.sysNetQuery:
    mov  eax, 49
    int  0x80
    ret

global main.sysOpenRaw
main.sysOpenRaw:
    mov  eax, 18
    int  0x80
    ret

global main.sysReadRaw
main.sysReadRaw:
    mov  eax, 19
    int  0x80
    ret

global main.sysWriteRaw
main.sysWriteRaw:
    mov  eax, 20
    int  0x80
    ret

global main.sysCloseRaw
main.sysCloseRaw:
    mov  eax, 21
    int  0x80
    ret

global main.sysProcInfoRaw
main.sysProcInfoRaw:
    mov  eax, 28
    int  0x80
    ret

global main.sysSchedSetRaw
main.sysSchedSetRaw:
    mov  eax, 29
    int  0x80
    ret

global main.sysFsyncRaw
main.sysFsyncRaw:
    mov  eax, 30
    int  0x80
    ret

global main.sysSocketOpenRaw
main.sysSocketOpenRaw:
    mov  eax, 31
    int  0x80
    ret

global main.sysSocketBindRaw
main.sysSocketBindRaw:
    mov  eax, 32
    int  0x80
    ret

global main.sysSocketListenRaw
main.sysSocketListenRaw:
    mov  eax, 33
    int  0x80
    ret

global main.sysSocketConnectRaw
main.sysSocketConnectRaw:
    mov  eax, 34
    int  0x80
    ret

global main.sysSocketAcceptRaw
main.sysSocketAcceptRaw:
    mov  eax, 35
    int  0x80
    ret

global main.sysSocketSendRaw
main.sysSocketSendRaw:
    mov  eax, 36
    int  0x80
    ret

global main.sysSocketRecvRaw
main.sysSocketRecvRaw:
    mov  eax, 37
    int  0x80
    ret

global main.sysSocketCloseRaw
main.sysSocketCloseRaw:
    mov  eax, 38
    int  0x80
    ret

global main.sysNetIfConfigRaw
main.sysNetIfConfigRaw:
    mov  eax, 39
    int  0x80
    ret

global main.sysNetRouteAddRaw
main.sysNetRouteAddRaw:
    mov  eax, 40
    int  0x80
    ret

global main.sysIsolationConfigRaw
main.sysIsolationConfigRaw:
    mov  eax, 41
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
    ; Atomic bump: tasks can be preempted mid-allocation, so the
    ; read-modify-write must be one instruction. xadd returns the old
    ; pointer (the allocation start) in rdi; the cell is seeded by _start.
    add  rdi, 7
    and  rdi, ~7
    lock xadd qword [abs GO_HEAP_PTR], rdi
    mov  rax, rdi
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
