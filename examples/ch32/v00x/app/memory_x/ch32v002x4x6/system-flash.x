/* CH32V002 application memory layout for system-flash bootloader (16K flash, 4K RAM).
 *
 * The application occupies all user flash minus the last page (boot metadata).
 * The bootloader lives in the separate system flash region.
 */
MEMORY
{
    RAM  : ORIGIN = 0x20000000, LENGTH = 4K

    /* Execution mirror of APP */
    CODE : ORIGIN = 0x00000000, LENGTH = 16K - 256

    /* Physical flash addresses */
    BOOT : ORIGIN = 0x1FFF0000, LENGTH = 3K + 256
    APP  : ORIGIN = 0x08000000, LENGTH = 16K - 256
    META : ORIGIN = 0x08000000 + 16K - 256, LENGTH = 256
}

/* qingke-rt expects a FLASH region */
REGION_ALIAS("FLASH", CODE);

REGION_ALIAS("REGION_TEXT", CODE);
REGION_ALIAS("REGION_RODATA", CODE);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
