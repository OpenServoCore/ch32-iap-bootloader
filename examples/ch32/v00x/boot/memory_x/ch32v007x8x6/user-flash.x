/* CH32V007 user-flash bootloader memory layout (62K flash, 8K RAM).
 *
 * The bootloader occupies the first 2 KB of user flash.
 * Boot metadata occupies the last page (256 bytes) of user flash.
 *
 * Flash map:
 *   0x0800_0000 .. 0x0800_07FF  bootloader   2 KB
 *   0x0800_0800 .. 0x0800_F6FF  application  60 KB - 256 B
 *   0x0800_F700 .. 0x0800_F7FF  boot meta    256 B
 */
MEMORY
{
    RAM  : ORIGIN = 0x20000000, LENGTH = 8K - 4   /* __tb_run_mode */

    /* Execution mirror of BOOT */
    CODE : ORIGIN = 0x00000000, LENGTH = 2K

    /* Physical flash addresses */
    BOOT : ORIGIN = 0x08000000, LENGTH = 2K
    APP  : ORIGIN = 0x08000000 + 2K, LENGTH = 60K - 256
    META : ORIGIN = 0x08000000 + 62K - 256, LENGTH = 256
}
