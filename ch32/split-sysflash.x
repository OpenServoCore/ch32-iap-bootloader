/* Split system-flash: places .text2 into the second system-flash region
 * (CODE2/BOOT2).  Only linked when the memory layout defines those regions. */
SECTIONS
{
    .text2 : ALIGN(4)
    {
        *(.text2 .text2.*);
    } > CODE2 AT> BOOT2
}
INSERT AFTER .rodata;
