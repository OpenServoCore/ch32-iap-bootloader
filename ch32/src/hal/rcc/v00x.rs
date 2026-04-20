pub fn enable_gpio(port_index: usize) {
    // IOPxEN starts at bit 2: IOPA=2, IOPB=3, IOPC=4, IOPD=5.
    ch32_metapac::RCC
        .pb2pcenr()
        .modify(|w| w.0 |= 1 << (2 + port_index));
}

pub fn enable_afio() {
    ch32_metapac::RCC.pb2pcenr().modify(|w| w.set_afioen(true));
}

const USART1EN: u32 = 1 << 14;
const USART2EN: u32 = 1 << 13;

/// PB2 bit for USART `n`, 0 if not on PB2.
pub const fn usart_apb2_bit(n: u8) -> u32 {
    match n {
        1 => USART1EN,
        2 => USART2EN,
        _ => 0,
    }
}

pub fn enable_usart(n: u8) {
    let bit = usart_apb2_bit(n);
    if bit != 0 {
        ch32_metapac::RCC.pb2pcenr().modify(|w| w.0 |= bit);
    }
}

/// Set PB2 enables in one write. Safe only during init.
#[inline(always)]
pub fn enable_apb2(bits: u32) {
    ch32_metapac::RCC.pb2pcenr().write(|w| w.0 = bits);
}

/// Pulse-reset and disable all PB2 peripherals.
#[inline(always)]
pub fn reset_apb2() {
    let rcc = ch32_metapac::RCC;
    let enabled = rcc.pb2pcenr().read().0;
    rcc.pb2prstr().write(|w| w.0 = enabled);
    rcc.pb2prstr().write(|w| w.0 = 0);
    rcc.pb2pcenr().write(|w| w.0 = 0);
}
