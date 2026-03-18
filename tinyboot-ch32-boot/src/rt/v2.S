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

    # --- 3. Copy .data initial values from flash to RAM ---
    #
    # Initialized mutable globals (e.g. `static mut X: u32 = 42`) need their
    # initial values. The linker stores these values in flash (the LMA — Load
    # Memory Address), but the code accesses them in RAM (the VMA — Virtual
    # Memory Address). We copy from flash → RAM so the variables start correct.
    #
    # a0 = _sdata  → RAM destination start (VMA, defined in link.x)
    # a1 = _sidata → flash source start   (LMA, defined in link.x)
    # a2 = _edata  → RAM destination end   (VMA, defined in link.x)
    la      a0, _sdata
    la      a1, _sidata
    la      a2, _edata

    # Copy loop: while a0 < a2, copy one word (4 bytes) at a time.
1:
    bgeu    a0, a2, 2f      # if dest >= end, we're done → skip to label 2
    lw      a3, 0(a1)       # a3 = word from flash[a1]
    sw      a3, 0(a0)       # RAM[a0] = a3
    addi    a0, a0, 4       # dest += 4
    addi    a1, a1, 4       # src  += 4
    j       1b              # loop back to label 1
2:

    # --- 4. Zero the .bss section ---
    #
    # Uninitialized globals (e.g. `static mut Y: u32 = 0`) go in .bss.
    # The C/Rust spec guarantees these start as zero, but RAM powers up with
    # random values — so we explicitly zero the range [_sbss, _ebss).
    la      a0, _sbss       # a0 = start of .bss in RAM
    la      a1, _ebss       # a1 = end of .bss in RAM

    # Zero loop: while a0 < a1, store zero one word at a time.
3:
    bgeu    a0, a1, 4f      # if pos >= end, done → skip to label 4
    sw      zero, 0(a0)     # RAM[a0] = 0  (zero is hardwired register x0)
    addi    a0, a0, 4       # pos += 4
    j       3b              # loop back to label 3
4:

    # --- 5. Jump to Rust ---
    #
    # Everything the Rust runtime needs is now set up:
    #   GP is valid, SP is valid, .data has correct values, .bss is zeroed.
    #
    # `main` is the Rust function marked #[unsafe(export_name = "main")].
    # We use `j` (jump) not `call` because main() -> ! never returns.
    j       main
