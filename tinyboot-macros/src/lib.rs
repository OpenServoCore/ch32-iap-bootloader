#![no_std]

#[macro_export]
macro_rules! tb_trace {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::trace!($($arg)*)
    };
}

#[macro_export]
macro_rules! tb_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! tb_info {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! tb_warn {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! tb_error {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! tb_assert {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::assert!($($arg)*);
        #[cfg(not(feature = "defmt"))]
        debug_assert!($($arg)*)
    };
}
