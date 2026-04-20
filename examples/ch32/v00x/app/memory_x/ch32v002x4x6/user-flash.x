/* CH32V002 application memory layout for user-flash bootloader (16K flash, 4K RAM).
 *
 * Flash map:
 *   0x0000_0000 .. 0x0000_07FF  bootloader   2 KB
 *   0x0000_0800 .. 0x0000_3EFF  application  14 KB - 256 B
 *   0x0000_3F00 .. 0x0000_3FFF  boot meta    256 B
 */
MEMORY
{
    RAM  : ORIGIN = 0x20000000, LENGTH = 4K - 4   /* __tb_run_mode */

    /* Execution mirror of APP */
    CODE : ORIGIN = 0x00000000 + 2K, LENGTH = 14K - 256

    /* Physical flash addresses */
    BOOT : ORIGIN = 0x08000000, LENGTH = 2K
    APP  : ORIGIN = 0x08000000 + 2K, LENGTH = 14K - 256
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
