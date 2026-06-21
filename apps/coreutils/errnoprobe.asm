; errnoprobe: distinct errno codes proof (sys_errno, ABI id 62).
;
; The kernel ABI returns a single -1 sentinel on failure; rlibc therefore mapped
; EVERY error to EIO. Now well-defined failure paths stamp a per-task errno that
; sys_errno (id 62) returns, so two DIFFERENT failures yield two DIFFERENT codes:
;   open("/data/nope")  -> -1, errno = ENOENT (2)   (no such file)
;   read(99, ...)       -> -1, errno = EBADF  (9)   (bad file descriptor)
; The probe fails unless both calls fail, each errno matches, and the two differ.
;
; sys_open: rdi=path, rsi=flags, rdx=mode (id 18).  sys_read: rdi=fd, rsi=buf,
; rdx=len (id 19).  sys_errno: () -> code (id 62).

bits 64
default rel

section .text
global _start
_start:
    ; --- open a nonexistent /data file -> must fail with ENOENT ---
    lea  rdi, [rel path]
    xor  esi, esi              ; O_RDONLY
    xor  edx, edx              ; mode 0
    mov  eax, 18               ; SYS_OPEN
    int  0x80
    cmp  rax, -1
    jne  .fail                 ; it must NOT exist
    mov  eax, 62               ; SYS_ERRNO
    int  0x80
    mov  r12, rax              ; r12 = errno after open (expect ENOENT=2)
    cmp  r12, 2
    jne  .fail

    ; --- read from a bad fd -> must fail with EBADF ---
    mov  edi, 99               ; out-of-range fd
    lea  rsi, [rel rbuf]
    mov  edx, 1
    mov  eax, 19               ; SYS_READ
    int  0x80
    cmp  rax, -1
    jne  .fail
    mov  eax, 62               ; SYS_ERRNO
    int  0x80
    mov  r13, rax              ; r13 = errno after read (expect EBADF=9)
    cmp  r13, 9
    jne  .fail

    ; --- the two causes must be DISTINCT (not one collapsed EIO) ---
    cmp  r12, r13
    je   .fail

    ; --- a failure that does NOT stamp a code must leave errno CLEARED (0), not
    ;     the STALE EBADF from the read above. read(len>4096) returns -1 before any
    ;     errno stamp; the dispatch cleared errno on entry, so sys_errno must be 0. ---
    xor  edi, edi
    lea  rsi, [rel rbuf]
    mov  edx, 5000             ; len > 4096 -> -1, un-stamped
    mov  eax, 19               ; SYS_READ
    int  0x80
    cmp  rax, -1
    jne  .fail
    mov  eax, 62               ; SYS_ERRNO
    int  0x80
    cmp  rax, 0                ; must be cleared, NOT a stale 9 (EBADF)
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
path:    db "/data/nope", 0
rbuf:    times 8 db 0
okmsg:   db "ERRNO: distinct ok", 10
okmsg_len equ $ - okmsg
failmsg: db "ERRNO: FAIL", 10
failmsg_len equ $ - failmsg
