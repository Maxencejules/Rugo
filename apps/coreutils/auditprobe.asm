; auditprobe: security audit-log proof (full-os guide Part IV.10).
;
; An external app has the STORAGE capability only, so a sys_net_query (id 49,
; needs NETWORK) is denied and recorded in the kernel audit ring. The probe
; then reads the ring (sys_sysinfo op 7) and echoes it -> the denial event for
; this task appears. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    ; trigger a capability denial (no NETWORK cap) -> audited
    mov  edi, 1                 ; net_query op 1 (DHCP discover)
    mov  eax, 49
    int  0x80                   ; returns -1; the point is the audit record

    ; read the audit-log tail
    mov  edi, 7                 ; sys_sysinfo op 7 = audit read
    lea  rsi, [rel buf]
    mov  edx, 200
    mov  eax, 61
    int  0x80
    test rax, rax
    js   .fail
    jz   .fail
    ; echo the captured audit tail
    lea  rdi, [rel buf]
    mov  esi, eax
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
okmsg:   db "AUDITPROBE: ok", 10
okmsg_len equ $ - okmsg
failmsg: db "AUDITPROBE: FAIL", 10
failmsg_len equ $ - failmsg

section .bss
buf: resb 200
