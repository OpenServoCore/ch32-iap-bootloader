    # =========================================================================
    # Minimal bootloader runtime.
    #
    # Omits .data copy, .bss zeroing, and interrupt vector table to save space
    # — critical when the bootloader must fit in ~2 KB of system flash.
    #
    # Safe because the bootloader:
    #   - Has no initialized statics (no .data)
    #   - Has no zero-init statics (no .bss)
    #   - Never enables interrupts (polling only)
    #   - Only runs from power-on reset (SRAM is undefined anyway)
    #
    # Applications should use the full qingke-rt runtime instead, which
    # includes .data/.bss init.
    # =========================================================================

    .section .init, "ax"
    .global _start
    .align 2

_start:
    .option push
    .option norelax
    la      gp, __global_pointer$
    .option pop

    la      sp, _stack_top
    j       main
