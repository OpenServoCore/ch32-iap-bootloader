# tinyboot

Rust bootloader for resource-constrained microcontrollers. Fits in the CH32V003's 1920-byte system flash with full trial boot, CRC16 app validation, OB-based metadata, and version reporting — leaving the entire 16KB user flash for the application.

## Why tinyboot?

I built tinyboot for [OpenServoCore](https://github.com/OpenServoCore), where CH32V006-based servo boards need seamless firmware updates over the existing DXL TTL bus — no opening the shell, no debug probe, just flash over the same wire the servos already talk on.

The existing options didn't fit:

- **CH32 factory bootloader** — Fixed to 115200 baud on PD5/PD6 with no way to configure UART pins, baud rate, or TX-enable for RS-485. Uses a sum-mod-256 checksum that silently drops bad commands with no error response. No CRC verification, no trial boot, no boot state machine. See [ch32v003-bootloader-docs](https://github.com/basilhussain/ch32v003-bootloader-docs) for the reverse-engineered protocol details.

- **embassy-boot** — A well-designed bootloader, but requires ~8KB of flash. That's half the V003's 16KB user flash, and doesn't fit in system flash at all. Not practical for MCUs with 16-32KB total.

I took it as a challenge to fit a proper bootloader — with a real protocol, CRC16 validation, trial boot, and configurable transport — into the CH32V003's 1920-byte system flash. The key inspiration was [rv003usb](https://github.com/cnlohr/rv003usb) by cnlohr, whose software USB implementation includes a 1920-byte bootloader in system flash. That project proved it was possible to fit meaningful code in that space, and showed me that the entire 16KB of user flash could be left free for the application.

### Design approach

tinyboot is structured as a library, not a monolithic binary. The core logic and protocol are platform-agnostic crates; chip-specific details live in separate `ch32-*` crates. To build your bootloader, you create a small crate with a `main.rs` that wires up your pin configuration, baud rate, and flash layout — see the [examples](examples/ch32/) for exactly this. The same split applies on the app side: [`tinyboot-ch32-app`](tinyboot-ch32-app/) integrates into your application so it can confirm a successful boot and reboot into the bootloader on command, enabling fully remote firmware updates without physical access.

## Features

- **Tiny** — Fits in 1920 bytes of CH32V003 system flash, leaving all 16KB user flash for the application
- **CRC16 validation** — Every frame is CRC16-CCITT protected; app image is verified end-to-end after flashing
- **Trial boot** — New firmware gets a limited number of boot attempts; if the app doesn't confirm, the bootloader takes over automatically
- **Boot state machine** — Idle / Updating / Validating lifecycle tracked in option bytes with forward-only bit transitions (no erase needed for state advances)
- **Version reporting** — Boot and app versions packed into flash, queryable over the wire
- **Configurable transport** — The protocol runs over any `embedded_io::Read + Write` stream. The CH32 implementation supports UART with configurable pins, baud rate, and optional TX-enable for RS-485 / DXL TTL, but the core is transport-agnostic — USB, SPI, Bluetooth, or WiFi would work just as well
- **App-side integration** — The app can confirm a successful boot and request bootloader entry over the wire, enabling fully remote firmware updates without physical access
- **Library, not binary** — Build your bootloader by creating a small crate that wires up your specific hardware; the core logic is reusable across chips
- **Modular and portable** — Platform-agnostic core with four traits (`Transport`, `Storage`, `BootMetaStore`, `BootCtl`) that you implement for your MCU; the protocol, state machine, and CLI work unchanged

## Crates

| Crate | Description |
| ----- | ----------- |
| [`tinyboot`](tinyboot/) | Platform-agnostic bootloader core (protocol dispatcher, boot state machine, app validation) |
| [`tinyboot-protocol`](tinyboot-protocol/) | Wire protocol (frame format, CRC16, commands) |
| [`tinyboot-ch32-hal`](tinyboot-ch32-hal/) | Minimal CH32 HAL (flash, GPIO, USART, RCC) |
| [`tinyboot-ch32-boot`](tinyboot-ch32-boot/) | CH32 bootloader platform (storage, boot control, OB metadata) |
| [`tinyboot-ch32-app`](tinyboot-ch32-app/) | CH32 app-side boot client (confirm, request update) |
| [`tinyboot-cli`](tinyboot-cli/) | Host-side CLI for flashing firmware over UART |

## Examples

- [`examples/ch32/system-flash`](examples/ch32/system-flash/) — 1920-byte bootloader in system flash (production)
- [`examples/ch32/user-flash`](examples/ch32/user-flash/) — 4KB bootloader in user flash (development, with defmt)

## Rust Version

The workspace uses **edition 2024**.

- **Library crates and CLI** — stable Rust 1.85+
- **CH32 examples** (bootloader and app binaries) — nightly, for `-Zbuild-std` on `riscv32ec-unknown-none-elf`

## Getting Started

1. **Build your bootloader** — create a small crate with a `main.rs` that configures your pins, baud rate, and flash layout. See the [system-flash example](examples/ch32/system-flash/) for a complete working setup, or the [user-flash example](examples/ch32/user-flash/) for a development configuration with defmt logging.

2. **Flash the bootloader** to system flash using [wlink](https://github.com/ch32-rs/wlink):

   ```sh
   wlink flash --address 0x1FFFF000 target/riscv32ec-unknown-none-elf/release/boot
   ```

3. **Install the CLI** and flash your app over UART:

   ```sh
   cargo install tinyboot-cli
   tinyboot flash target/riscv32ec-unknown-none-elf/release/app --reset
   ```

## Porting to a New Chip

tinyboot's core crates (`tinyboot`, `tinyboot-protocol`, `tinyboot-cli`) are platform-agnostic. To add support for a new MCU family, you implement four traits and provide a minimal HAL. Here's what that looks like:

### 1. Create a HAL crate (`tinyboot-{chip}-hal`)

Provide the bare minimum register-level operations your bootloader needs:

- **Flash** — unlock, erase page, write halfword/word, lock
- **GPIO** — configure pin mode, set high/low (for TX-enable if using RS-485)
- **USART** — init with baud rate, blocking read byte, blocking write byte, flush
- **RCC/clock** — enable peripheral clocks
- **Reset** — system reset, and optionally jump-to-address for user-flash bootloaders

For CH32, we use [ch32-metapac](https://github.com/ch32-rs/ch32-data) for register definitions. For STM32, you could use [stm32-metapac](https://github.com/embassy-rs/stm32-data) or raw PAC crates. The HAL should be minimal — this code runs in a bootloader, not an application.

### 2. Create a boot crate (`tinyboot-{chip}-boot`)

Implement the four platform traits from `tinyboot::traits::boot`:

| Trait | What to implement |
| ----- | ----------------- |
| `Transport` | Any `embedded_io::Read + Write` stream — UART, RS-485, USB, SPI, even WiFi or Bluetooth. The protocol doesn't care what carries the bytes |
| `Storage` | Implement `embedded_storage::NorFlash` (erase, write) and provide `as_slice()` for zero-copy flash reads, plus `unlock()` |
| `BootMetaStore` | Read/write boot state, trial counter, and app checksum from your chip's equivalent of option bytes or a reserved flash page |
| `BootCtl` | `is_boot_requested()` checks your boot flag (OB bit, RAM magic, GPIO pin, etc.); `system_reset()` resets or jumps to app |

Wire them together in a `Platform` struct and pass it to `Core::new(platform).run()`.

### 3. Create an app crate (`tinyboot-{chip}-app`)

Implement `tinyboot::traits::app::BootClient`:

- `confirm()` — transition boot state from Validating back to Idle
- `request_update()` — set your boot request flag and reset into the bootloader

### What you get for free

The entire protocol (frame format, CRC, sync, commands), the boot state machine (Idle/Updating/Validating transitions, trial boot logic, app validation), the CLI, and the host-side flashing workflow all work unchanged. You only write the chip-specific glue.

## Current Status

tinyboot currently supports the **CH32V003** over **UART / RS-485**. This is tested and working end-to-end for both system-flash and user-flash configurations.

**What's coming:** I have dev boards on order for several other CH32 family chips (V006, V103, V203, X035, etc.) and plan to add support as they arrive. The architecture is already chip-agnostic — it's mostly a matter of adding HAL implementations and testing on real hardware.

**Want a different transport or chip?** [File an issue.](https://github.com/OpenServoCore/tinyboot/issues) USB support in particular would be a natural addition. Transports like Bluetooth are harder to fit in system flash, though newer CH32 chips with larger system flash regions may make it feasible — no guarantees.

## Contributing

Contributions are welcome — especially new chip ports and transport implementations. If you're thinking about adding support for a new MCU family, the [Porting to a New Chip](#porting-to-a-new-chip) section above covers the trait surface you'd need to implement.

Please [open an issue](https://github.com/OpenServoCore/tinyboot/issues) before starting a large PR so we can discuss the approach.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
