; arch/x86_64/isr.asm — ISR stubs for exceptions and IRQs
bits 64
default rel

section .text

extern trap_handler

; Macro for exceptions WITHOUT a CPU-pushed error code
%macro ISR_NOERR 1
global isr_stub_%1
isr_stub_%1:
    push qword 0           ; dummy error code
    push qword %1          ; vector number
    jmp isr_common
%endmacro

; Macro for exceptions WITH a CPU-pushed error code
%macro ISR_ERR 1
global isr_stub_%1
isr_stub_%1:
    push qword %1          ; vector number (error code already on stack)
    jmp isr_common
%endmacro

; --- Exception stubs ---
ISR_NOERR 0                ; #DE  Divide Error
ISR_NOERR 3                ; #BP  Breakpoint
ISR_ERR   8                ; #DF  Double Fault
ISR_ERR   13               ; #GP  General Protection Fault
ISR_ERR   14               ; #PF  Page Fault

; --- IRQ stubs (remapped by PIC to vectors 32+) ---
ISR_NOERR 32               ; IRQ0  PIT timer
ISR_NOERR 64               ; Native-driver MSI/MSI-X
ISR_NOERR 65               ; Local APIC spurious vector

; --- Software interrupt for syscalls (int 0x80 = vector 128) ---
ISR_NOERR 128              ; Syscall gate (DPL=3 set in IDT by Rust)

; --- Common tail: save all GPRs, call Rust handler, restore, iretq ---
;
; Stack layout on entry (before GPR push):
;   [RSP+0]   vector         (pushed by stub)
;   [RSP+8]   error code     (CPU or dummy 0)
;   [RSP+16]  RIP            (CPU interrupt frame)
;   [RSP+24]  CS
;   [RSP+32]  RFLAGS
;   [RSP+40]  RSP (old)
;   [RSP+48]  SS
;
; After GPR push, RSP points to full InterruptFrame passed to trap_handler.
isr_common:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    mov  rdi, rsp           ; arg1: pointer to InterruptFrame
    call trap_handler

    pop  r15
    pop  r14
    pop  r13
    pop  r12
    pop  r11
    pop  r10
    pop  r9
    pop  r8
    pop  rbp
    pop  rdi
    pop  rsi
    pop  rdx
    pop  rcx
    pop  rbx
    pop  rax

    add  rsp, 16            ; remove vector + error_code
    iretq

section .note.GNU-stack noalloc noexec nowrite progbits
