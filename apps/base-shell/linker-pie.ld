/* PIE (ET_DYN) variant of the base-shell linker (full-os Part IV.10, code-base ASLR).
 * Linked -shared so the kernel can load it at a RANDOM base every spawn. The base_shell
 * code is `default rel` (RIP-relative), so it has no relocations -- it runs correctly at
 * any base. Coreutils stay ET_EXEC (linker.ld). */
ENTRY(_start)

SECTIONS
{
    . = 0;

    .text : { *(.text .text.*) }
    .rodata : { *(.rodata .rodata.*) }
    .dynamic : { *(.dynamic) }
    .rela.dyn : { *(.rela.dyn) }

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
}
