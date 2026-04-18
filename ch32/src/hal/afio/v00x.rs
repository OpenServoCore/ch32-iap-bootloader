#[inline(always)]
pub fn set_usart_remap(n: u8, remap: u8) {
    if remap == 0 {
        return;
    }
    let afio = ch32_metapac::AFIO;
    match n {
        1 => afio.pcfr1().modify(|w| w.set_usart1_rm(remap)),
        2 => afio.pcfr1().modify(|w| w.set_usart2_rm(remap)),
        _ => {}
    }
}
