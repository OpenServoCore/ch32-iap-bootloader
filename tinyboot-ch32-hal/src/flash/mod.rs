#[cfg_attr(flash_v0, path = "v0.rs")]
#[cfg_attr(flash_v1, path = "v1.rs")]
mod family;

pub use family::*;

// OB address constants — used by boot crate's BootMetaStore and app crate's confirm.
pub const OB_BASE: u32 = 0x1FFFF800;
pub const META_OB_BASE: u32 = OB_BASE + 16;
