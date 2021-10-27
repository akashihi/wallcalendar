#![deny(missing_docs)]
#![deny(unsafe_code)]

//! Converts BIN images into WallCalendar flash file format
//!
//! Usage:
//! `bin2flash [--small] <input>` - will scan images directory and pack the into flash images.
//! Parameters `--small` will enforce using just a first two months data for all images data
//! to produce flash blob that will fit into MCU for debug purposes.
//!
//! Image directory is expected to have particular subdirectories with particular files in them:
//!
//! -- /big_digits/{0..9}.bin - Mandatory, contains big digits (80x48)
//! -- /data/%mm%-%dd%-a-black.bin - Mandatory, frontpage of calendar data in black color (480x420)
//! -- /data/%mm%-%dd%-a-red.bin - Optional, red channel of calendar frontpage (480x420)
//! -- /data/%mm%-%dd%-b-black.bin - Mandatory, other side of calendar data page in black color (480x420)
//! -- /months/{apr,aug..sep}.bin - Mandatory, month names (168x28)
//! -- /moon/moon{1..8}.bin - Mandatory, moon phases, with full moon at moon1 (16x1)
//! -- /small_digits/{0..9}.bin - Mandatory, contains big digits (16x16)
//! -- /weekdays/{fri,mon..wed}.bin - Mandatory, weekdays names (192x30)
//!
//! Format specification:
//! File begins with directory of entries. Each entry is u32, pointing to the first byte of
//! `.bin` file related to entry. First 8 bits of entry are unused, but could be used in the future for 8-bit checksum.
//! Offset `0x0000` is a marker of missing data. Entries follow each other in the following order:
//! * 10 entries of big digits
//! * 10 entries of small digits
//! * 12 month names entries
//! * 7 weekdays entries
//! * 8 moon phase entries
//! * 366 triplets of a side black, a side red and b side red images for each day, starting from 1st of January
//!
//!The directory takes 366*3 + 8 + 7 + 12 + 10 +10 = 1145 entries or 1145*32=36640 bytes. First images starts
//! exactly after directory
//!

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use byteorder::{BigEndian, WriteBytesExt};
use clap::{Parser};
use humansize::{file_size_opts as options, FileSize};
use anyhow::Result;

#[macro_use]
extern crate log;

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Denis Chaplygin <akashihi@gmail.com>")]
struct Opts {
    /// Input directory
    input: String,
    /// Generate smaller flash image for debug purposes
    #[clap(short, long)]
    simple: bool,
}

fn add_file_to_flash(fname: &Path, offset: u32, entries_directory: &mut[u32], directory_index: usize, output_file: &mut File) -> Result<u32> {
    info!("{}, size: {}", fname.as_os_str().to_str().unwrap(),
            fname.metadata().unwrap().len().
            file_size(options::CONVENTIONAL).unwrap_or_else(|_| "Unknown".to_string()));
    let bytes = std::fs::read(fname).unwrap();
    entries_directory[directory_index] = offset;
    output_file.write_all(&bytes).unwrap();
    Ok(offset + bytes.len() as u32)
}

fn dump_directory(output_file: &mut File, entries_directory: &[u32]) -> Result<u32> {
    let mut offset = 0;
    output_file.seek(SeekFrom::Start(0))?;
    for entry in entries_directory {
        output_file.write_u32::<BigEndian>(*entry)?;
        offset += 4;
    };
    Ok(offset)
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    pretty_env_logger::init();

    let opts: Opts = Opts::parse();
    info!("Input directory: {}", opts.input);

    let mut entries_directory: [u32; 1145] = [0; 1145];
    let mut directory_index=0;

    let mut output_file = OpenOptions::new().write(true).create(true).truncate(true).open("spiflash.bin").unwrap();
    let mut offset = dump_directory(&mut output_file, &entries_directory).unwrap();

    //Add Big Digits
    for i in 0..10 {
        let digit_fname: PathBuf = [&opts.input, "big_digits", &format!("{}.bin", i)].iter().collect();
        offset = add_file_to_flash(&digit_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
        directory_index+=1;
    }

    //Add Small Digits
    for i in 0..10 {
        let digit_fname: PathBuf = [&opts.input, "small_digits", &format!("{}.bin", i)].iter().collect();
        offset = add_file_to_flash(&digit_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
        directory_index+=1;
    }

    //Add months
    for name in ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"] {
        let month_fname: PathBuf = [&opts.input, "months", &format!("{}.bin", name)].iter().collect();
        offset = add_file_to_flash(&month_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
        directory_index+=1;
    }

    //Add weekdays
    for name in ["mon", "tue", "wed", "thu", "fri", "sat", "sun"] {
        let weekday_fname: PathBuf = [&opts.input, "weekdays", &format!("{}.bin", name)].iter().collect();
        offset = add_file_to_flash(&weekday_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
        directory_index+=1;
    }

    //Add moon phases
    for i in 1..9 {
        let moon_fname: PathBuf = [&opts.input, "moon", &format!("moon{}.bin", i)].iter().collect();
        offset = add_file_to_flash(&moon_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
        directory_index+=1;
    }

    //Add data pages
    for m in 1..13 {
        for d in 1..32 {
            if (m == 4 || m== 6 || m==9 || m==11) && d>30 {
                continue
            }
            if m==2 && d>29 {
                continue
            }
            let a_black_fname: PathBuf = [&opts.input, "data", &format!("{:02}-{:02}-a-black.bin", m, d)].iter().collect();
            offset = add_file_to_flash(&a_black_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
            directory_index+=1;


            let a_red_fname: PathBuf = [&opts.input, "data", &format!("{:02}-{:02}-a-red.bin", m, d)].iter().collect();
            if a_red_fname.exists() {
                offset = add_file_to_flash(&a_red_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
                directory_index+=1;
            } else {
                entries_directory[directory_index] = 0;
                directory_index+=1;
            }

            let b_fname: PathBuf = [&opts.input, "data", &format!("{:02}-{:02}-b.bin", m, d)].iter().collect();
            offset = add_file_to_flash(&b_fname, offset, &mut entries_directory, directory_index, &mut output_file).unwrap();
            directory_index+=1;
        }
    }

    dump_directory(&mut output_file, &entries_directory).unwrap();

}
