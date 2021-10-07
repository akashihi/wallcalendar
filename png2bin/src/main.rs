#![deny(missing_docs)]
#![deny(unsafe_code)]

//! Converts PNG images into WallCalendar image format
//!
//! Usage:
//! `png2bin <dir>` - will conert each _png_ file in _dir_ into bin file
//!
//! Format specification:
//! * First two bytes - length of the encoded data in bytes, little endian
//! * data - compressed 1BPP image.
//!
//! Image is scanned from left to right, top to bottom. Each pixel
//! is represented as a bit in a data stream and will be set to 1 for
//! while pixel and 0 for any other color. In case amount of pixels is not
//! dividable by 8, missing pixels will be stuffed with value 1. Resulting
//! bitstream will be compressed using LSZZ algorithm.
//!
//! Pay attention, that we do not store image dimension, as they are standardized
//! and are well known by the firmware.

use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "1.0", author = "Denis Chaplygin <akashihi@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    // Input directory
    input: String,
}
fn main() {
    let opts: Opts = Opts::parse();
    println!("Input directory: {}", opts.input)
}
