use embedded_graphics::{prelude::*};
use embedded_graphics::primitives::Rectangle;
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

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error> where D: DrawTarget<Color=Self::Color> {
        target.draw_iter(StripePixels{current_pixel: 0})
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error> where D: DrawTarget<Color=Self::Color> {
        todo!()
    }
}