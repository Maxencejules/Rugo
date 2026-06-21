; loginprobe: multi-user authenticated login proof (full-os guide Part IV.10).
;
; sys_proc_ctl (id 51) op 5 = login(rsi=name[8], rdx=pw): verifies a credential
; against the password database and, on success, assumes that account's uid. An
; external app starts as uid 100; a wrong password is denied (uid unchanged), and
; the correct root password elevates to uid 0. console-write=0 exit=2.

bits 64
default rel

section .text
global _start
_start:
    ; getuid -> 100 (external apps run as the regular user)
    mov  edi, 3
    mov  eax, 51
    int  0x80
    cmp  rax, 100
    jne  .fail
    ; login root / WRONG password -> denied (-1)
    mov  edi, 5
    lea  rsi, [rel uname]
    lea  rdx, [rel badpw]
    mov  eax, 51
    int  0x80
    cmp  rax, -1
    jne  .fail
    ; uid unchanged after a failed login
    mov  edi, 3
    mov  eax, 51
    int  0x80
    cmp  rax, 100
    jne  .fail
    ; login root / CORRECT password -> uid 0
    mov  edi, 5
    lea  rsi, [rel uname]
    lea  rdx, [rel goodpw]
    mov  eax, 51
    int  0x80
    test rax, rax
    jnz  .fail
    ; getuid now 0 (authenticated privilege change took effect)
    mov  edi, 3
    mov  eax, 51
    int  0x80
    test rax, rax
    jnz  .fail
    lea  rdi, [rel okmsg]
    mov  esi, okmsg_len
    xor  eax, eax
    int  0x80
    ; --- lockout proof: LOGIN_LOCKOUT (3) consecutive wrong root logins lock
    ; the account, after which even the correct password is refused ---
    mov  r12d, 3
.lock_bad:
    mov  edi, 5
    lea  rsi, [rel uname]
    lea  rdx, [rel badpw]
    mov  eax, 51
    int  0x80
    cmp  rax, -1
    jne  .fail              ; each wrong password must be denied
    dec  r12d
    jnz  .lock_bad
    ; account locked now: the CORRECT password is also refused
    mov  edi, 5
    lea  rsi, [rel uname]
    lea  rdx, [rel goodpw]
    mov  eax, 51
    int  0x80
    cmp  rax, -1
    jne  .fail              ; locked -> correct password still denied
    lea  rdi, [rel lockmsg]
    mov  esi, lockmsg_len
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
uname:  db "root", 0, 0, 0, 0          ; 8-byte username field
badpw:  db "wrong", 0                   ; NUL-terminated wrong password
goodpw: db "toor", 0                    ; NUL-terminated correct root password
okmsg:   db "LOGINPROBE: ok", 10
okmsg_len equ $ - okmsg
lockmsg: db "LOGINPROBE: lockout ok", 10
lockmsg_len equ $ - lockmsg
failmsg: db "LOGINPROBE: FAIL", 10
failmsg_len equ $ - failmsg
