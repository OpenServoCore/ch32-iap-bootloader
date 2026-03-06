#![no_std]
#![no_main]

use panic_halt as _;

use ch32_iap_bootloader::Bootloader;
use qingke_rt::entry;

#[entry]
fn main() -> ! {
    Bootloader::default().run();
}
