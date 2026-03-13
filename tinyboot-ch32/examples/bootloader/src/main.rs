#![no_std]
#![no_main]

#[cfg(feature = "defmt")]
use defmt_rtt as _;

use tinyboot_ch32::boot::Bootloader;

#[unsafe(export_name = "main")]
fn main() -> ! {
    Bootloader::default().run();
}
