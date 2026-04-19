//! Example app for the tinyboot bootloader (CH32V00x).
//!
//! - TIM2 interrupt blinks LED on PD0 at 1 Hz.
//! - Main loop listens on USART1 (Remap3: TX=PC0, RX=PC1) and reboots into
//!   the bootloader when it receives a Reset command.

#![no_std]
#![no_main]

mod transport;

use core::cell::RefCell;

use ch32_hal::gpio::{Level, Output};
use ch32_hal::interrupt::InterruptExt;
use ch32_hal::pac;
use ch32_hal::time::Hertz;
use ch32_hal::timer::low_level::Timer;
use ch32_hal::usart::{self, Uart};
use critical_section::Mutex;

use defmt_rtt as _;

tinyboot_ch32::app::app_version!();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    defmt::error!("panic");
    loop {}
}

type Shared<T> = Mutex<RefCell<Option<T>>>;
static LED: Shared<Output<'static>> = Mutex::new(RefCell::new(None));

fn invert_level(level: Level) -> Level {
    match level {
        Level::High => Level::Low,
        Level::Low => Level::High,
    }
}

#[qingke_rt::entry]
fn main() -> ! {
    let p = ch32_hal::init(Default::default());

    // LED blink via TIM2 interrupt (2 Hz toggle = 1 Hz blink)
    critical_section::with(|cs| {
        LED.borrow_ref_mut(cs)
            .replace(Output::new(p.PD0, Level::Low, Default::default()));
    });
    let tim = Timer::new(p.TIM2);
    tim.set_frequency(Hertz::hz(2));
    tim.enable_update_interrupt(true);
    tim.start();
    unsafe { ch32_hal::interrupt::TIM2.enable() };

    // USART1 blocking — must match the bootloader's pin mapping.
    // ch32-hal generic param picks the remap:
    //   0 (default): TX=PD5, RX=PD6
    //   1: TX=PD6, RX=PD5
    //   2: TX=PD0, RX=PD1
    //   3: TX=PC0, RX=PC1
    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 115200;
    let uart = Uart::new_blocking::<3>(p.USART1, p.PC1, p.PC0, uart_config).unwrap();
    let (tx, rx) = uart.split();
    let mut rx = transport::Rx(rx);

    // RS-485 DE/RE on PC2, matching the bootloader. tx_level=Low means idle-RX
    // drives the pin High (inverse), which tri-states U4A on the V006 dev board
    // so LinkE UART TX can reach MCU_RX without contention.
    let tx_level = Level::Low;
    let tx_en_pin = Output::new(p.PC2, invert_level(tx_level), Default::default());
    let mut tx = transport::Tx {
        uart: tx,
        tx_en: Some(transport::TxEn {
            pin: tx_en_pin,
            tx_level,
        }),
    };

    let mut app = tinyboot_ch32::app::new_app(tinyboot_ch32::app::BootCtl::new());
    app.confirm();

    defmt::info!("Boot confirmed, app ready.");

    loop {
        app.poll(&mut rx, &mut tx);
    }
}

#[qingke_rt::interrupt]
fn TIM2() {
    pac::TIM2.intfr().modify(|w| w.set_uif(false));
    critical_section::with(|cs| {
        if let Some(ref mut led) = *LED.borrow_ref_mut(cs) {
            led.toggle();
            if led.is_set_high() {
                defmt::info!("LED on");
            } else {
                defmt::info!("LED off");
            }
        }
    });
}
