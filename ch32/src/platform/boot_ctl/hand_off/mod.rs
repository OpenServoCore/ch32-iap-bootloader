//! App hand-off strategies.
//!
//! Exactly one variant compiles per build, selected by the `system-flash` feature:
//! - [`system`]: software reset; factory ROM re-reads the boot request and dispatches.
//! - [`user`]: reset APB2 peripherals, then jump directly to the app's reset vector.

#[cfg(feature = "system-flash")]
mod system;
#[cfg(not(feature = "system-flash"))]
mod user;

#[cfg(feature = "system-flash")]
pub type Active = system::SystemHandOff;
#[cfg(not(feature = "system-flash"))]
pub type Active = user::UserHandOff;
