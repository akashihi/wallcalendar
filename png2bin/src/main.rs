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
//! is represented as a bit in a data stream and will be set to 0 for
//! black pixel and 1 for any other color. In case amount of pixels is not
//! dividable by 8, missing pixels will be stuffed with value 1. Resulting
//! bitstream will be compressed using LSZZ algorithm.
//!
//! Pay attention, that we do not store image dimension, as they are standardized
//! and are well known by the firmware.

use clap::{AppSettings, Clap};
use std::fs::{DirEntry, File, OpenOptions};
use humansize::{FileSize, file_size_opts as options};
use png::{ColorType, BitDepth};
use bit_field::BitField;
use lzss::{Lzss, SliceReader, VecWriter};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

#[macro_use]
extern crate log;

#[derive(Clap)]
#[clap(version = "1.0", author = "Denis Chaplygin <akashihi@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    // Input directory
    input: String,
}

type MyLzss = Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;

fn compress_image(input: DirEntry) {
    let basename = input.path().file_name().and_then(|f| f.to_str()).map(|s| s.to_owned()).unwrap();

    let decoder = png::Decoder::new(File::open(input.path()).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let image = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..image.buffer_size()];

    if image.color_type != ColorType::Grayscale || image.bit_depth != BitDepth::Eight {
        panic!("{}: Only 8-bit grayscale images are supported! ({:?}, {:?})", basename, image.color_type, image.bit_depth)
    }

    let width = image.width;
    let height = image.height;

    let bitstream_size = if (width*height/8)%8 == 0 {
        (width*height/8) as usize
    } else {
        (width*height/8+1) as usize
    };
    let mut bitstream = Vec::with_capacity(bitstream_size);
    bitstream.resize(bitstream_size, 0xFF_u8);

    for byte in 0..bitstream_size {
        for bit in 0..8 {
            let image_index = byte*8+bit;
            bitstream[byte].set_bit(bit, bytes[image_index]>0);
        }
    }
    let compressed = VecWriter::with_capacity(32_768);
    let compressed_bytes = MyLzss::compress(SliceReader::new(&bitstream), compressed).unwrap();

    let mut output_filename = input.path().clone();
    output_filename.set_extension("bin");
    let mut output = OpenOptions::new().create(true).truncate(true).write(true).open(output_filename).unwrap();
    output.write_u16::<LittleEndian>(compressed_bytes.len() as u16).unwrap();
    output.write_all(&compressed_bytes).unwrap();
    output.flush().unwrap();

    let file_size = input.path().metadata().unwrap().len();
    info!("{} {}x{}, PNG: {}, BIN: {}, compressed: {}", basename, width, height, file_size.file_size(options::CONVENTIONAL).unwrap(), bitstream_size.file_size(options::CONVENTIONAL).unwrap(), compressed_bytes.len().file_size(options::CONVENTIONAL).unwrap())
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();
    info!("Input directory: {}", opts.input);

    let files = std::fs::read_dir(opts.input).unwrap()
        .filter(|f|
            f.as_ref().map(|name| name.file_name().to_str()
                .map(|s| s.to_lowercase())
                .map(|s| s.ends_with(".png"))
                .unwrap_or(false))
                .unwrap_or(false))
        .collect::<Result<Vec<_>, std::io::Error>>().unwrap();
    for file in files {
        compress_image(file)
    }
}
