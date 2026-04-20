/* CH32V002 system-flash bootloader memory layout (16K flash, 4K RAM).
 *
 * The bootloader runs from system flash at 0x1FFF0000.
 * All 16 KB of user flash is available for the application.
 * Boot metadata occupies the last page (256 bytes) of user flash.
 */
MEMORY
{
    RAM  : ORIGIN = 0x20000000, LENGTH = 4K

    /* Execution mirror of BOOT */
    CODE : ORIGIN = 0x00000000, LENGTH = 3K + 256

    /* Physical flash addresses */
    BOOT : ORIGIN = 0x1FFF0000, LENGTH = 3K + 256
    APP  : ORIGIN = 0x08000000, LENGTH = 16K - 256
    META : ORIGIN = 0x08000000 + 16K - 256, LENGTH = 256
}
