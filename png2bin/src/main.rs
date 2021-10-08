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
use png::{ColorType, BitDepth, OutputInfo};
use bit_field::BitField;
use lzss::{Lzss, SliceReader, VecWriter};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;
use anyhow::{Result, Context};
use thiserror::Error;

#[macro_use]
extern crate log;

#[derive(Error, Debug)]
enum ConversionError {
    #[error("Path can not be converted to string")]
    UnprocessablePath,
    #[error("File is not a Grayscale image")]
    NotGrayscale,
    #[error("File is not a 8-bit image")]
    NotEightBit
}

#[derive(Clap)]
#[clap(version = "1.0", author = "Denis Chaplygin <akashihi@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    // Input directory
    input: String,
}

type MyLzss = Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;

fn validate_image(image: &OutputInfo) -> Result<bool> {
    if image.color_type != ColorType::Grayscale {
        Err(ConversionError::NotGrayscale.into())
    } else if image.bit_depth != BitDepth::Eight {
        Err(ConversionError::NotEightBit.into())
    } else {
        Ok(true)
    }
}

fn allocate_bitstream(image: &OutputInfo) -> Vec<u8> {
    let image_size = image.width*image.height;
    let bitstream_size = if image_size%8 == 0 {
        (image_size/8) as usize
    } else {
        (image_size/8+1) as usize
    };
    let mut bitstream = Vec::with_capacity(bitstream_size);
    bitstream.resize(bitstream_size, 0xFF_u8);
    bitstream
}

fn read_png(input: &DirEntry) -> Result<(Vec<u8>, OutputInfo)> {
    let decoder = png::Decoder::new(File::open(input.path())?);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let image = reader.next_frame(&mut buf)?;
    buf.truncate(image.buffer_size());
    Ok((buf, image))
}

fn write_bin(input: &DirEntry, data: &[u8]) -> Result<()> {
    let mut output_filename = input.path().clone();
    output_filename.set_extension("bin");
    let mut output = OpenOptions::new().create(true).truncate(true).write(true).open(output_filename)?;
    output.write_u16::<LittleEndian>(data.len() as u16)?;
    output.write_all(&data)?;
    output.flush().context("Bin output")
}

fn compress_image(input: DirEntry) -> Result<()>{
    let basename = input.path().file_name().and_then(|f| f.to_str()).map(|s| s.to_owned()).ok_or(ConversionError::UnprocessablePath)?;

    let (bytes, image) = read_png(&input)?;

    validate_image(&image)?;
    let mut bitstream= allocate_bitstream(&image);

    for byte in 0..bitstream.len() {
        for bit in 0..8 {
            let image_index = byte*8+bit;
            bitstream[byte].set_bit(bit, bytes[image_index]>0);
        }
    }
    let compressed = VecWriter::with_capacity(32_768);
    let compressed_bytes = MyLzss::compress(SliceReader::new(&bitstream), compressed)?;

    write_bin(&input, &compressed_bytes)?;

    let file_size = input.path().metadata()?.len();
    info!("{} {}x{}, PNG: {}, BIN: {}", basename, image.width, image.height, file_size.file_size(options::CONVENTIONAL).unwrap_or("Unknown".to_string()), compressed_bytes.len().file_size(options::CONVENTIONAL).unwrap_or("Unknown".to_string()));
    Ok(())
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();
    info!("Input directory: {}", opts.input);

    std::fs::read_dir(opts.input).unwrap()
        .filter(|f|
            f.as_ref().map(|name| name.file_name().to_str()
                .map(|s| s.to_lowercase())
                .map(|s| s.ends_with(".png"))
                .unwrap_or(false))
                .unwrap_or(false))
        .map(|file| file.context("File access").and_then(compress_image))
        .filter(|r| r.is_err())
        .for_each(|e| error!("{}", e.err().unwrap()));
}
