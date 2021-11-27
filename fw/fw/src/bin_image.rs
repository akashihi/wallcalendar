use embedded_graphics::{prelude::*};
use embedded_graphics::primitives::Rectangle;
use epd_waveshare::prelude::TriColor;
use bit_field::BitField;

pub struct BinImage<'a> {
    size: Size,
    bw_plane: &'a [u8],
    rw_plane: Option<&'a [u8]>,
}

impl<'a> BinImage<'a> {
    pub fn from_slice(size: Size, bw_plane: &'a [u8], rw_plane: Option<&'a [u8]>) -> BinImage<'a> {
        BinImage{size, bw_plane, rw_plane}
    }
}

impl OriginDimensions for BinImage<'_> {
    fn size(&self) -> Size {
        self.size
    }
}

impl ImageDrawable for BinImage<'_> {
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
    image: &'a BinImage<'a>
}

impl<'a> BinImageIterator<'a> {
    pub fn new(image: &'a BinImage<'a>) -> BinImageIterator<'a> {
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
            let color = self.image.rw_plane.map(|rw_plane| rw_plane[bit_pixel_offset].get_bit(bit_pixel_position))
                .map(|is_red| if ! is_red { TriColor::Chromatic} else { bw_color}).unwrap_or(bw_color);

            let result = Some(Pixel(Point::new(x as i32, y as i32), color));
            self.current_pixel += 1;
            result
        } else {
            None
        }
    }
}