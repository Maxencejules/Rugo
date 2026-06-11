/* External app linked into the exec app window [0x140_0000, 0x180_0000). */
ENTRY(_start)

SECTIONS
{
    . = 0x1400000;

    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
}
