    # =========================================================================
    # WARNING: This is NOT a normal RISC-V startup.
    #
    # This is a stripped-down bootloader runtime that omits several things
    # a normal application startup must do. Do not copy this for general use.
    #
    # What is missing and why:
    #
    # 1. NO INTERRUPT VECTOR TABLE
    #    The bootloader never enables interrupts. It runs a synchronous
    #    polling loop (UART RX → process → UART TX) with no ISRs. The
    #    CH32V003 PFIC is left in its reset state. A normal application
    #    would need a vector table at 0x0 (or wherever PFIC.VTFADDR points).
    #
    # 2. NO .data SECTION INIT (flash → RAM copy)
    #    The bootloader has no initialized mutable statics (no `static mut`,
    #    no `static X: Mutex<Cell<T>>`, etc.). All state lives on the stack
    #    or in hardware registers. A normal application with any non-zero
    #    initialized static would get garbage values without this copy.
    #
    # 3. NO .bss SECTION INIT (RAM zeroing)
    #    Same reason — no zero-initialized statics exist. A normal
    #    application relying on zero-init statics would read uninitialized
    #    RAM without this zeroing.
    #
    # These omissions save 68 bytes of flash — critical when the entire
    # bootloader must fit in 1920 bytes of system flash.
    # =========================================================================

    # Place this code in the .init section, which link.x puts at the very
    # start of flash (address 0). This is the first thing the CPU executes.
    .section .init, "ax"
    .global _start
    .align 2

_start:
    # --- 1. Initialize the Global Pointer (GP) ---
    #
    # RISC-V uses GP-relative addressing to access small globals in a single
    # instruction instead of two (lui+lw → just lw). The compiler emits these
    # GP-relative accesses for any variable within ±2KB of __global_pointer$.
    #
    # We must use `norelax` here: the linker's "relaxation" pass tries to
    # shorten instructions by assuming GP is already set — but we're the ones
    # setting it! Without norelax, the linker would turn `la gp, __global_pointer$`
    # into a nop (since it thinks GP already equals __global_pointer$), and GP
    # would remain garbage.
    .option push
    .option norelax

    la      gp, __global_pointer$   # gp = address of __global_pointer$ (defined in link.x)

    .option pop

    # --- 2. Initialize the Stack Pointer (SP) ---
    #
    # The stack grows downward, so SP starts at the top of RAM.
    # _stack_top is defined in link.x as ORIGIN(RAM) + LENGTH(RAM).
    # On CH32V003: 0x20000000 + 2048 = 0x20000800.
    la      sp, _stack_top

    # --- 3. Jump to Rust ---
    #
    # Everything the Rust runtime needs is now set up:
    #   GP is valid, SP is valid, .data and .bss are skipped because we don't use them.
    #
    # `main` is the Rust function marked #[unsafe(export_name = "main")].
    # We use `j` (jump) not `call` because main() -> ! never returns.
    j       main
