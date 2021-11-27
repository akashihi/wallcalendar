use embedded_graphics::{prelude::*};
use embedded_graphics::primitives::Rectangle;
use epd_waveshare::prelude::TriColor;

pub struct StripePixels {
    current_pixel: i32
}

impl Iterator for StripePixels {
    type Item = Pixel<TriColor>;

    fn next(&mut self) -> Option<Self::Item> {
        let colors : [TriColor; 19] = [TriColor::Black, TriColor::Chromatic, TriColor::White,TriColor::Black, TriColor::Chromatic, TriColor::White,
            TriColor::Black, TriColor::Chromatic, TriColor::White,TriColor::Black, TriColor::Chromatic, TriColor::White,
            TriColor::Black, TriColor::Chromatic, TriColor::White,TriColor::Black, TriColor::Chromatic, TriColor::White, TriColor::White];
        if self.current_pixel < 648*480 {
            let x = self.current_pixel%480;
            let y = self.current_pixel/480;
            let color = colors[(y/36) as usize];
            let result = Some(Pixel(Point::new(x, y), color));
            self.current_pixel += 1;
            result
        } else {
            None
        }
    }
}

pub struct SmallSquarePixels {
    current_pixel: i32
}

impl Iterator for SmallSquarePixels {
    type Item = Pixel<TriColor>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_pixel < 100*100 {
            let result = Some(Pixel(Point::new(self.current_pixel%100, self.current_pixel/100), TriColor::Chromatic));
            self.current_pixel += 1;
            result
        } else {
            None
        }
    }
}

pub struct BinImage {
    select: bool
}

impl BinImage {
    pub fn new_stripes() -> Self {
        BinImage{select: true}
    }

    pub fn new_square() -> Self {
        BinImage{select: false}
    }
}

impl OriginDimensions for BinImage {
    fn size(&self) -> Size {
        if self.select {
            Size::new(480, 648)
        } else {
            Size::new(100, 100)
        }
    }
}

impl ImageDrawable for BinImage {
    type Color = TriColor;

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error> where D: DrawTarget<Color=Self::Color> {
        if self.select {
            target.draw_iter(StripePixels{current_pixel: 0})
        } else {
            target.draw_iter(SmallSquarePixels{current_pixel: 0})
        }
    }

    fn draw_sub_image<D>(&self, target: &mut D, area: &Rectangle) -> Result<(), D::Error> where D: DrawTarget<Color=Self::Color> {
        todo!()
    }
}