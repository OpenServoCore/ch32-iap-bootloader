/* CH32V002 user-flash bootloader memory layout (16K flash, 4K RAM).
 *
 * The bootloader occupies the first 2 KB of user flash.
 * Boot metadata occupies the last page (256 bytes) of user flash.
 *
 * Flash map:
 *   0x0800_0000 .. 0x0800_07FF  bootloader   2 KB
 *   0x0800_0800 .. 0x0800_3EFF  application  14 KB - 256 B
 *   0x0800_3F00 .. 0x0800_3FFF  boot meta    256 B
 */
MEMORY
{
    RAM  : ORIGIN = 0x20000000, LENGTH = 4K - 4   /* __tb_run_mode */

    /* Execution mirror of BOOT */
    CODE : ORIGIN = 0x00000000, LENGTH = 2K

    /* Physical flash addresses */
    BOOT : ORIGIN = 0x08000000, LENGTH = 2K
    APP  : ORIGIN = 0x08000000 + 2K, LENGTH = 14K - 256
    META : ORIGIN = 0x08000000 + 16K - 256, LENGTH = 256
}
