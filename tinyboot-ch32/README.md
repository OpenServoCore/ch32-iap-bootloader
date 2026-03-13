# tinyboot-ch32

CH32V003 platform support for tinyboot. Provides UART/RS-485 transport, flash storage, and boot state management.

## Binary size

The bootloader compiles to ~1,888 bytes (release, `opt-level="z"`, LTO). This fits in the CH32V003's system flash (boot area) with room to spare.

## Flash targets

There are two ways to deploy the bootloader:

### User flash (default)

The bootloader lives in the first 2 pages (2 KB) of user flash at `0x08000000`. The app occupies the remaining flash. This is the default when building with the `memory-x` feature.

**Pros:** No special tooling required, works with probe-rs.
**Cons:** Costs 2 flash pages from your app space.

### System flash (boot area) — recommended

The bootloader replaces the WCH factory bootloader in system flash at `0x1FFFF000`. The entire 16 KB of user flash is available for the app.

The CH32V003 system flash layout:

| Address | Size | Contents |
|---|---|---|
| `0x1FFFF000` | 3,328 B | System flash (BOOT_3KB+256B) |
| `0x1FFFF700` | 256 B | Vendor bytes (factory-locked) |
| `0x1FFFF800` | 256 B | Option bytes |

Tinyboot uses ~1,888 of the 3,328 available bytes, leaving ~1,440 bytes of headroom.

#### Flashing to system flash

To flash to system flash, disable the auto-generated memory layout and provide a custom `memory.x`:

```
MEMORY
{
    FLASH : ORIGIN = 0x1FFFF000, LENGTH = 3328
    RAM   : ORIGIN = 0x20000000, LENGTH =    2K
}

REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
```

In `Cargo.toml`, disable the `memory-x` default feature:

```toml
tinyboot-ch32 = { path = "../..", features = ["ch32v003f4p6", "bootloader"], default-features = false }
```

In `build.rs`, add the linker search path:

```rust
let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
println!("cargo:rustc-link-search={dir}");
```

#### Flashing with probe-rs (recommended)

With a custom probe-rs build that includes the boot flash algorithm (see [probe-rs boot flash support](#probe-rs-boot-flash-support) below), flashing is automatic:

```sh
cargo build --release
probe-rs download --chip CH32V003 target/riscv32ec-unknown-none-elf/release/bootloader
```

probe-rs reads `p_paddr` (physical/load address) from the ELF program headers. When `memory.x` sets `FLASH ORIGIN = 0x1FFFF000`, the ELF segments have `paddr = 0x1FFFF000`, so probe-rs automatically routes the data to the boot flash algorithm. No `--base-address` flag needed.

For raw binaries, specify the address manually:

```sh
rust-objcopy -O binary target/riscv32ec-unknown-none-elf/release/bootloader bootloader.bin
probe-rs download --chip CH32V003 --binary-format bin --base-address 0x1FFFF000 bootloader.bin
```

#### Flashing with wlink

[wlink](https://github.com/ch32-rs/wlink) also supports boot flash — specify the address explicitly:

```sh
cargo build --release
rust-objcopy -O binary target/riscv32ec-unknown-none-elf/release/bootloader bootloader.bin
wlink flash --address 0x1FFFF000 --chip CH32V003 bootloader.bin
```

#### Legacy memory.x compatibility

Projects using the default `memory-x` feature get `FLASH ORIGIN = 0x00000000` (the memory-mapped alias). This still works for user flash — the existing probe-rs flash algorithm translates `0x0`-based addresses to the physical `0x08000000` address internally. No changes needed for existing user flash workflows.

#### Recovery

If the bootloader is broken, the chip is still recoverable via the SWD/SDI debug interface — the debug probe works regardless of what is in the boot flash. Use `wlink` to reflash.

#### Restoring the factory bootloader

Back up the original boot flash before overwriting:

```sh
wlink dump 0x1FFFF000 1920 factory-bootloader.bin
```

Restore later with:

```sh
wlink flash --address 0x1FFFF000 --chip CH32V003 factory-bootloader.bin
```

## CH32V003 system flash reference

The system flash (boot area) at `0x1FFFF000` is **not OTP** — it is regular flash that can be erased and reprogrammed via the debug interface. It is protected by a dedicated lock mechanism separate from the user flash lock.

### Flash controller registers

| Register | Address | Description |
|---|---|---|
| `FLASH_KEYR` | `0x40022004` | User flash unlock |
| `FLASH_OBKEYR` | `0x40022008` | Option byte unlock |
| `FLASH_STATR` | `0x4002200C` | Flash status |
| `FLASH_CTLR` | `0x40022010` | Flash control |
| `FLASH_ADDR` | `0x40022014` | Target address |
| `FLASH_OBR` | `0x4002201C` | Option byte readback |
| `FLASH_WPR` | `0x40022020` | Write protection |
| `FLASH_MODEKEYR` | `0x40022024` | Fast mode unlock |
| `FLASH_BOOT_MODEKEYP` | `0x40022028` | **Boot area unlock** |

All key registers use the same two-key unlock sequence:

```
KEY1 = 0x45670123
KEY2 = 0xCDEF89AB
```

### FLASH_BOOT_MODEKEYP (0x40022028)

This write-only register unlocks the boot flash area for programming. Writing KEY1 followed by KEY2 clears the `BOOT_LOCK` bit (bit 15) in `FLASH_STATR`, allowing erase and program operations to target the `0x1FFFF000` region.

After reset, `BOOT_LOCK` defaults to 1 (locked). The boot area cannot be erased or programmed from user code running on the MCU — it can only be programmed via the external debug interface (SDI/SWD).

### FLASH_STATR (0x4002200C)

| Bit | Name | Description |
|---|---|---|
| 0 | `BSY` | Flash busy |
| 4 | `WRPRTERR` | Write protection error |
| 5 | `EOP` | End of operation |
| 14 | `MODE` | Boot mode — 0: boot from user flash, 1: boot from system flash |
| 15 | `BOOT_LOCK` | Boot area lock — 0: unlocked, 1: locked (default) |

### FLASH_CTLR (0x40022010)

| Bit | Name | Description |
|---|---|---|
| 0 | `PG` | Page program |
| 1 | `PER` | Page erase |
| 2 | `MER` | Mass erase |
| 4 | `OPTPG` | Option byte program |
| 5 | `OPTER` | Option byte erase |
| 6 | `STRT` | Start operation |
| 7 | `LOCK` | Flash lock (cleared by `FLASH_KEYR` unlock) |
| 15 | `FLOCK` | Fast mode lock (cleared by `FLASH_MODEKEYR` unlock) |
| 16 | `PAGE_PG` | Fast page program |
| 17 | `PAGE_ER` | Fast page erase |
| 18 | `BUF_LOAD` | Load word into page buffer |
| 19 | `BUF_RST` | Reset page buffer |

### Unlock sequence for boot flash programming

To program the boot area via the debug interface:

```
1. Unlock user flash:
   Write FLASH_KEYR     = 0x45670123  (KEY1)
   Write FLASH_KEYR     = 0xCDEF89AB  (KEY2)

2. Unlock fast mode:
   Write FLASH_MODEKEYR = 0x45670123  (KEY1)
   Write FLASH_MODEKEYR = 0xCDEF89AB  (KEY2)

3. Unlock boot area:
   Write FLASH_BOOT_MODEKEYP = 0x45670123  (KEY1)
   Write FLASH_BOOT_MODEKEYP = 0xCDEF89AB  (KEY2)

4. Fast page erase (64 bytes per page):
   Set CTLR.PAGE_ER
   Write FLASH_ADDR = target page address
   Set CTLR.STRT
   Poll STATR.BSY until clear
   Clear CTLR.PAGE_ER

5. Fast page program (64 bytes per page):
   Set CTLR.PAGE_PG
   Set CTLR.BUF_RST, poll BSY
   For each 4-byte word:
     Write word to target address
     Set CTLR.BUF_LOAD, poll BSY
   Write FLASH_ADDR = page base address
   Set CTLR.STRT, poll BSY
   Clear CTLR.PAGE_PG

6. Re-lock:
   Set CTLR.FLOCK
   Set CTLR.LOCK
```

### How wlink programs boot flash

wlink does not directly write flash controller registers. Instead, it uploads a pre-compiled 498-byte flash algorithm binary to MCU SRAM via the WCH-Link USB protocol. This SRAM-resident code performs the actual flash controller manipulation. Data is streamed in 1024-byte chunks over USB.

Stock probe-rs does not support the boot flash region — see [probe-rs boot flash support](#probe-rs-boot-flash-support) below for how to add it.

### Boot mode selection from user code

The app can request a reboot into the bootloader (system flash) by setting the `MODE` bit:

```
1. Unlock boot mode:
   Write FLASH_BOOT_MODEKEYP = 0x45670123  (KEY1)
   Write FLASH_BOOT_MODEKEYP = 0xCDEF89AB  (KEY2)

2. Set MODE bit (FLASH_STATR bit 14):
   Write FLASH_STATR |= (1 << 14)

3. Clear all reset flags:
   Write RCC_RSTSCKR |= (1 << 24)   (RMVF — clears latched flags)

4. Software reset:
   Write PFIC_CFGR = 0xBEEF0080     (KEYCODE + RESETSYS)
```

Step 3 is critical — the factory bootloader checks that **only** the software reset flag (`SFTRSTF`) is set. Without clearing stale flags first, leftover power-on reset flags cause the bootloader to immediately bounce back to user code.

## probe-rs boot flash support

Stock probe-rs (as of v0.31) does not know about the CH32V003 boot flash region. We have a working proof-of-concept that adds support.

### What was changed

**1. Flash algorithm** (`ch32v003-boot`)

A modified version of the existing `ch32v003` user flash algorithm with three changes:

- **`FLASH_BOOT_MODEKEYP` unlock** in `Init()` — writes KEY1/KEY2 to `0x40022028` to clear `BOOT_LOCK`, allowing writes to the `0x1FFFF000` region.
- **64-byte page erase** — uses `PAGE_ER` (fast page erase) instead of `PER` (1KB sector erase), since boot flash is only 3,328 bytes.
- **No address fixup** — the user flash algorithm adds `0x08000000` to addresses because `0x0` is a memory-mapped alias. Boot flash addresses (`0x1FFFF000`) are already physical.

`program_page` is unchanged — both user and boot flash use the same 64-byte `PAGE_PG` (fast page program) mechanism.

**2. Target YAML** (`CH32V0_Series.yaml`)

Added to the CH32V003 chip definition:
- NVM region: `0x1FFFF000` – `0x1FFFFD00` (3,328 bytes)
- Flash algorithm entry: `ch32v003-boot` with 64-byte sectors

### How probe-rs flash algorithms work

1. Flash algorithms are small binaries (~500 bytes) written in Rust using the [`flash-algorithm`](https://github.com/probe-rs/flash-algorithm) crate. They implement `Init`, `EraseSector`, `ProgramPage`, and `EraseChip` via the `FlashAlgorithm` trait.
2. The binary is compiled for the target arch (riscv32ec), then `target-gen elf` extracts it and base64-encodes it into a YAML snippet.
3. The YAML goes into `probe-rs/targets/CH32V0_Series.yaml` and is baked into the probe-rs binary at compile time.
4. At runtime, probe-rs uploads the algorithm to target RAM (`0x20000020`), then calls entry points by setting the PC register and resuming the core via the debug interface.
5. probe-rs selects the correct algorithm by matching the target address against `flash_properties.address_range` in each algorithm entry.

### How probe-rs auto-detects boot vs user flash

probe-rs reads `p_paddr` (physical address) from ELF program headers to determine where to flash. When the linker script sets `FLASH ORIGIN = 0x1FFFF000`, the ELF has `paddr = 0x1FFFF000`, which matches the boot flash NVM region. No flags needed — `probe-rs download bootloader.elf` just works.

For user flash, the default `memory.x` sets `FLASH ORIGIN = 0x0`, producing `paddr = 0x0`, which matches the user flash NVM region. The flash algorithm internally translates `0x0` → `0x08000000` (the physical address the flash controller expects).

#### VMA/LMA split for boot flash

When booting from system flash, the hardware remaps `0x1FFFF000` to `0x00000000`. Code _executes_ at `0x0` (VMA) but must be _flashed_ to `0x1FFFF000` (LMA). Ideally the ELF would encode both:

```
p_vaddr = 0x00000000   (execution address — where the CPU sees the code)
p_paddr = 0x1FFFF000   (load address — where probe-rs should flash it)
```

This requires a custom linker script with `AT()` to set LMA separately from VMA:

```
MEMORY
{
    BOOT : ORIGIN = 0x1FFFF000, LENGTH = 3328
    RAM  : ORIGIN = 0x20000000, LENGTH =    2K
}

SECTIONS
{
    .text 0x00000000 : AT(ORIGIN(BOOT))
    {
        *(.init);
        *(.text .text.*);
    } > BOOT

    /* .rodata, .data, etc. follow the same pattern */
}
```

This produces an ELF where code addresses start at `0x0` (correct for execution in boot mode) but `p_paddr = 0x1FFFF000` (so probe-rs flashes to the right place).

**Practical workaround:** riscv-rt's standard `link.x` does not support `AT()` overrides via `memory.x` alone. The simpler approach is to set `FLASH ORIGIN = 0x1FFFF000` — code is linked at `0x1FFFF000`, which works because the CH32V003 maps system flash at _both_ `0x0` and `0x1FFFF000` simultaneously. Addresses in the `0x1FFFF000` range are always accessible regardless of boot mode.

```
/* Simple approach — works because both addresses map to the same flash */
MEMORY
{
    FLASH : ORIGIN = 0x1FFFF000, LENGTH = 3328
    RAM   : ORIGIN = 0x20000000, LENGTH =    2K
}

REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
```

Both VMA and LMA are `0x1FFFF000`. probe-rs sees `paddr = 0x1FFFF000` and routes to the boot flash algorithm automatically. The code runs correctly because `0x1FFFF000` is always a valid address for system flash.

A proper VMA/LMA split would require either a custom `link.x` or upstream changes to riscv-rt / ch32-metapac's memory.x generation to support `AT>` regions for boot flash.

### Build process

```
flash algorithm source (Rust)
    ↓ cargo build --release (riscv32ec target)
ELF binary
    ↓ target-gen elf
YAML snippet (base64 blob + entry points + flash properties)
    ↓ paste into CH32V0_Series.yaml
probe-rs build
    ↓ cargo build -p probe-rs-tools
probe-rs binary with boot flash support
```

## Plan: unified CH32 flash algorithm

The current state of CH32 flash support in probe-rs is fragmented — separate contributors submitted algorithms for V003, V208, V307, and F1, each built with different PACs and toolchains. The V003 algorithm uses the outdated `ch32v0` PAC (v0.1.7) and a no-longer-available `riscv32ec` target.

### Goal

A single flash algorithm crate using [`ch32-metapac`](https://github.com/ch32-rs/ch32-data) that generates `CH32V0_Series.yaml` (and potentially V2, V3, X, L series) covering:

- All V0 chips: V002, V003, V004, V005, V006, V007
- Both user flash and boot flash regions
- Correct flash sizes and RAM sizes from metapac metadata

### Why ch32-metapac

- Covers all WCH chips from one crate (53+ chip features)
- Has `FLASH_BOOT_MODEKEYP` properly defined
- Includes chip metadata (flash sizes, RAM sizes, peripheral addresses)
- Actively maintained via [ch32-rs/ch32-data](https://github.com/ch32-rs/ch32-data)

### Algorithm design

One binary handles both user and boot flash by branching on address:

```rust
fn erase_sector(&mut self, mut addr: u32) -> Result<(), ErrorCode> {
    if addr >= 0x1FFFF000 {
        // Boot flash: 64B page erase, no address fixup
        rb.ctlr.modify(|w| w.set_page_er(true));
    } else {
        // User flash: 1KB sector erase, translate 0x0 → 0x08000000
        if addr < 0x08000000 { addr += 0x08000000; }
        rb.ctlr.modify(|w| w.set_per(true));
    }
    ...
}
```

The `Init` function always unlocks `FLASH_BOOT_MODEKEYP` — this is harmless when targeting user flash.

### YAML generation

A build script reads chip metadata from `ch32-metapac` and generates per-chip variants:

```yaml
variants:
- name: CH32V003    # 16KB flash, 2KB RAM
- name: CH32V006    # 62KB flash, 8KB RAM
- name: CH32V007    # 62KB flash, 8KB RAM
# ... etc
```

Each variant gets:
- User flash NVM region (size from metapac)
- Boot flash NVM region at `0x1FFFF000`
- RAM region (size from metapac)
- References to both `ch32v0` and `ch32v0-boot` algorithm entries

### Upstream path

1. Create algorithm source repo (e.g. `OpenServoCore/ch32-flash-algorithms`)
2. PR to `probe-rs/probe-rs` — update `CH32V0_Series.yaml` with generated YAML
3. Bonus: PR to `ch32-rs/ch32-data` — update metapac's `memory.x` generation to emit `FLASH ORIGIN = 0x1FFFF000` for boot flash configurations, enabling the ELF paddr auto-detection

### References

- [probe-rs/probe-rs](https://github.com/probe-rs/probe-rs) — Debug probe toolkit
- [probe-rs/flash-algorithm](https://github.com/probe-rs/flash-algorithm) — Flash algorithm framework crate
- [ch32-rs/flash-algorithms](https://github.com/ch32-rs/flash-algorithms) — Existing CH32 algorithms (outdated, unbuildable with current toolchains)
- [ch32-rs/ch32-data](https://github.com/ch32-rs/ch32-data) — ch32-metapac source and chip metadata
- [CH32V003 Reference Manual V1.9](https://ch32-riscv-ug.github.io/CH32V003/datasheet_en/CH32V003RM.PDF) — Chapter 16 (Flash), pages 174-185
- [basilhussain/ch32v003-bootloader-docs](https://github.com/basilhussain/ch32v003-bootloader-docs) — Reverse-engineered factory bootloader documentation
- [ch32-rs/wlink](https://github.com/ch32-rs/wlink) — Open-source WCH-Link flash tool (Rust)
- [cnlohr/rv003usb](https://github.com/cnlohr/rv003usb) — USB bootloader that replaces factory bootloader in system flash
- [monte-monte/ch32_user_bootloader_flasher](https://github.com/monte-monte/ch32_user_bootloader_flasher) — Bootloader replacement tool with backup/restore
