use core::marker::PhantomData;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::Pixel;
use embedded_graphics::prelude::{ImageDrawable, Point};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::LineHeight::Pixels;
use epd_waveshare::prelude::TriColor;

pub struct StripePixels {
    current_pixel: i32
}

impl Iterator for StripePixels {
    type Item = Pixel<TriColor>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pixel < 648/480 {
            let result = Some(Pixel(Point::new(self.current_pixel%480, self.current_pixel/480), TriColor::Black));
            self.current_pixel += 1;
            result
        } else {
            None
        }
    }
}

pub struct BinImage;

impl BinImage {
    pub fn new_stripes() -> Self {
        BinImage
    }
}

impl OriginDimensions for BinImage {
    fn size(&self) -> Size {
        Size::new(648, 480)
    }
}

impl ImageDrawable for BinImage {
    type Color = TriColor;

    fn draw<D>(&self, target: &mut D) -> Result<(), embedded_graphics_core::draw_target::Error> where D: DrawTarget<Color=Self::Color> {
        target.draw_iter()
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), embedded_graphics_core::draw_target::Error> where D: DrawTarget<Color=Self::Color> {
        todo!()
    }
}