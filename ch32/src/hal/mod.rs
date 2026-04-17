//! Chip-level hardware abstraction (flash, gpio, usart, …).

mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
pub use generated::{Pin, UsartMapping};

pub mod afio;
pub mod flash;
pub mod gpio;
pub mod iwdg;
pub mod pfic;
pub mod rcc;
pub mod usart;

pub mod boot_request;
