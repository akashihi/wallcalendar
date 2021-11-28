use embedded_graphics::{prelude::*};
use embedded_graphics::primitives::Rectangle;
use epd_waveshare::prelude::TriColor;
use bit_field::BitField;
use alloc::vec::Vec;
use lzss::{Lzss, SliceReader, SliceWriter};

type MyLzss = Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;

pub struct BinImage {
    size: Size,
    bw_plane: Vec<u8>,
    rw_plane: Option<Vec<u8>>,
}

fn inflate(size: Size, data: &[u8]) -> Vec<u8>{
    let plane_size = (size.width/8 * size.height) as usize;
    let mut plane = Vec::with_capacity(plane_size);
    plane.resize(plane_size, 0x00);
    let mut reader = SliceReader::new(&data);
    let mut writer = SliceWriter::new(&mut plane);
    MyLzss::decompress(reader, writer).unwrap();
    plane
}

impl BinImage {
    pub fn from_slice(size: Size, bw_data: &[u8], rw_data: Option<& [u8]>) -> BinImage {
        let bw_plane = inflate(size, bw_data);
        let rw_plane = rw_data.map(|data| inflate(size, data));
        BinImage{size, bw_plane, rw_plane}
    }
}

impl OriginDimensions for BinImage {
    fn size(&self) -> Size {
        self.size
    }
}

impl ImageDrawable for BinImage {
    type Color = TriColor;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error> where D: DrawTarget<Color=Self::Color> {
        target.draw_iter(BinImageIterator::new(self))
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error> where D: DrawTarget<Color=Self::Color> {
        todo!()
    }
}

pub struct BinImageIterator<'a> {
    current_pixel: u32,
    image: &'a BinImage
}

impl<'a> BinImageIterator<'a> {
    pub fn new(image: &'a BinImage) -> BinImageIterator<'a> {
        BinImageIterator{ current_pixel: 0, image}
    }
}

impl<'a> Iterator for BinImageIterator<'a> {
    type Item = Pixel<TriColor>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pixel < self.image.size.width*self.image.size.height {
            let x = self.current_pixel%self.image.size.width;
            let y = self.current_pixel/self.image.size.width;
            let bit_pixel_offset = (self.current_pixel/8) as usize;
            let bit_pixel_position = (self.current_pixel%8) as usize;
            let bw_color = if self.image.bw_plane[bit_pixel_offset].get_bit(bit_pixel_position) {
                TriColor::White
            } else {
                TriColor::Black
            };
            let color = self.image.rw_plane.as_ref().map(|rw_plane| rw_plane[bit_pixel_offset].get_bit(bit_pixel_position))
                .map(|is_red| if ! is_red { TriColor::Chromatic} else { bw_color}).unwrap_or(bw_color);

            let result = Some(Pixel(Point::new(x as i32, y as i32), color));
            self.current_pixel += 1;
            result
        } else {
            None
        }
    }
}