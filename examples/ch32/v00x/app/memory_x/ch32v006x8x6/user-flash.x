/* CH32V006 application memory layout for user-flash bootloader (62K flash, 8K RAM).
 *
 * Flash map:
 *   0x0000_0000 .. 0x0000_07FF  bootloader   2 KB
 *   0x0000_0800 .. 0x0000_F6FF  application  60 KB - 256 B
 *   0x0000_F700 .. 0x0000_F7FF  boot meta    256 B
 */
MEMORY
{
    RAM  : ORIGIN = 0x20000000, LENGTH = 8K - 4   /* __tb_run_mode */

    /* Execution mirror of APP */
    CODE : ORIGIN = 0x00000000 + 2K, LENGTH = 60K - 256

    /* Physical flash addresses */
    BOOT : ORIGIN = 0x08000000, LENGTH = 2K
    APP  : ORIGIN = 0x08000000 + 2K, LENGTH = 60K - 256
    META : ORIGIN = 0x08000000 + 62K - 256, LENGTH = 256
}

/* qingke-rt expects a FLASH region */
REGION_ALIAS("FLASH", CODE);

REGION_ALIAS("REGION_TEXT", CODE);
REGION_ALIAS("REGION_RODATA", CODE);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
