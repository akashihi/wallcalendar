#![deny(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

//! Celestial calculations support
//!
//! * weekday
//! * sunrise/sunset
//! * moon phase

mod moon;
mod sun;
mod weekday;

pub use moon::moon_phase as moon_phase;
pub use weekday::weekday as weekday;
pub use sun::sunrise as sunrise;
pub use sun::sunset as sunset;