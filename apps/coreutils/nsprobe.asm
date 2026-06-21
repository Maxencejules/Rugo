; nsprobe: PID namespace isolation proof (sys_nsctl, ABI id 57).
;
; The defining property of a PID namespace: a task that unshares into a new
; namespace sees an ISOLATED process view (only itself, until it clones) and a
; namespace-LOCAL pid starting at 1 (it becomes its namespace's "init"). This
; proves it from ring 3, single client:
;   op 2 ns_task_count BEFORE unshare -> the whole system (>1, services running)
;   op 1 unshare_pid                  -> a fresh namespace id
;   op 2 ns_task_count AFTER          -> 1 (only itself is in the new namespace)
;   op 3 ns_getpid                    -> 1 (namespace-local pid, not its global tid)
; Fails unless the global view had >1 task and the namespaced view is exactly 1
; with local pid 1.
;
; sys_nsctl: rdi=op (id 57).

bits 64
default rel

section .text
global _start
_start:
    mov  edi, 2                 ; ns_task_count (global view, before unshare)
    mov  eax, 57
    int  0x80
    mov  r12, rax               ; G = global live-task count
    cmp  r12, 1
    jbe  .fail                  ; must be >1 (boot task + services + shell + us)

    mov  edi, 1                 ; unshare_pid -> fresh namespace
    mov  eax, 57
    int  0x80
    cmp  rax, -1
    je   .fail

    mov  edi, 2                 ; ns_task_count (now namespace-isolated)
    mov  eax, 57
    int  0x80
    cmp  rax, 1                 ; only this task is in the new namespace
    jne  .fail

    mov  edi, 3                 ; ns_getpid -> namespace-local pid
    mov  eax, 57
    int  0x80
    cmp  rax, 1                 ; first task in a fresh namespace is pid 1 (init)
    jne  .fail

    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax
    int  0x80

    ; --- UTS namespace: the hostname is namespace-scoped ---
    mov  edi, 5                 ; gethostname (this namespace, none set yet)
    mov  eax, 57
    int  0x80
    mov  rbx, 0x6F677572        ; "rugo" little-endian (the global default)
    cmp  rax, rbx
    jne  .fail                  ; a fresh namespace inherits the global hostname

    mov  edi, 4                 ; sethostname("ctr") for this namespace
    lea  rsi, [rel hostname]
    mov  edx, 3
    mov  eax, 57
    int  0x80
    cmp  rax, 0
    jne  .fail

    mov  edi, 5                 ; gethostname again
    mov  eax, 57
    int  0x80
    mov  rbx, 0x00727463        ; "ctr" little-endian
    cmp  rax, rbx
    jne  .fail                  ; the namespace now has its OWN hostname

    lea  rdi, [rel utsmsg]
    mov  esi, utsmsg_len
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
hostname: db "ctr"
okmsg:   db "NS: pid-namespace isolated ok", 10
okmsg_len equ $ - okmsg
utsmsg:  db "NS: uts-namespace hostname ok", 10
utsmsg_len equ $ - utsmsg
failmsg: db "NS: FAIL", 10
failmsg_len equ $ - failmsg
