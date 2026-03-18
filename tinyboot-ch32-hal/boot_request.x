/* Reserves 4 bytes at the start of RAM for the boot request word.
 * Shared between bootloader and app. NOLOAD so it is not zeroed
 * on startup, preserving the value across soft resets.
 *
 * Apps using tinyboot with user-flash mode should include this
 * linker script: -Tboot_request.x */
SECTIONS
{
    .boot_request ORIGIN(RAM) (NOLOAD) :
    {
        __boot_request = .;
        . += 4;
    } > RAM
} INSERT BEFORE .data;
