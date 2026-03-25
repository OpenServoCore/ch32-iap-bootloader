# Changelog

## [0.2.1] - 2026-03-25

### Fixed

- **UB in boot metadata reads** — fixed memory alignment issue by using `u32` buffer and casting back to `u8` array
- **App version display** — fixed `app_version` read and app boot for user-flash example
- **mtvec for apps behind bootloader** — `qingke-rt` hardcodes `mtvec = 0x0`, breaking interrupts in apps loaded at non-zero addresses; added `fix_mtvec!` macro to `tinyboot-ch32-app` that wraps `_setup_interrupts` via linker `--wrap` to rewrite `mtvec` to the actual vector table base
- **Peripheral cleanup before app jump** — properly reset APB2 peripherals (`rcc::reset_apb2`) before jumping to app, preventing stale peripheral state from leaking into the application
- **defmt panics on app→bootloader reset** — split bootloader runtime into `v2.S` (minimal, no .data/.bss init) and `v2_full.S` (full init for defmt); the `defmt` feature selects the appropriate startup

### Added

- **CLI logging** — `env_logger` support; set `RUST_LOG=debug` for protocol-level diagnostics

### Optimized

- ~180 bytes saved in system-flash bootloader via aggressive inlining, CRC/payload merge, batched RCC enable, custom `read_exact`/`write_all` overrides, and boot version cleanup
- All CH32V003 chip variants added with CI coverage

## [0.2.0] - 2026-03-20

### Changed

- **Breaking:** Verify command now carries `app_size` in the addr field
- **Breaking:** `BootMetaStore` trait: `trials_remaining()` replaced by `has_trials() -> bool`; `refresh()` takes an additional `app_size` parameter
- **Breaking:** `BootMetaStore::new()` replaced by `Default` impl (`BootMetaStore::default()`)
- **Breaking:** `BootCtl::system_reset()` takes `BootMode` enum (`App` / `Bootloader`) instead of `bool`
- CRC16 validation now covers only actual firmware bytes, not the entire flash region
- CLI only writes actual firmware data — no more 0xFF padding to fill the region
- App version read from end of binary (`flash[app_size-2..app_size]`) instead of end of flash region
- Linker script places `.tinyboot_version` after all other flash content (end of binary) instead of at end of flash region
- OB metadata expanded from 8 to 16 bytes (added app_size u32 field)
- System flash memory.x corrected to LENGTH=1920 (actual system flash size)

### Added

- `iwdg::feed()` in HAL — feeds the independent watchdog timer before OB erase in app-side `confirm()` to prevent watchdog reset during the critical OB erase+rewrite window
- `BootMode` enum (`App` / `Bootloader`) — replaces bare `bool` in boot control APIs
- `has_trials() -> bool` on `BootMetaStore` trait — simpler and avoids software popcount on targets without hardware support

### Optimized

- Startup assembly stripped to 20 bytes (from 88) — removed .data copy loop, .bss zero loop, and alignment padding since the bootloader uses no mutable statics
- Flash time reduced proportionally to firmware size (e.g. 5KB app on 16KB chip: ~8s vs full-region flash)
- CRC verification faster — only covers firmware bytes

## [0.1.0] - 2026-03-20

Initial release.
