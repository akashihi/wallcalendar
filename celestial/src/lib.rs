#![deny(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

//! Celestial calculations support
//!
//! * weekday
//! * sunrise/sunset
//! * moon phase

mod weekday;

pub use weekday::weekday as weekday;