use celestial::{day_of_the_year, moon_phase, sunrise, sunset};
use embedded_graphics::image::Image;
use embedded_graphics::prelude::*;
use epd_waveshare::epd5in83b_v2::Display5in83;
use board::hal::datetime::Date;
use crate::{BinImage, ImageManager, Watch};
use crate::holiday::is_holiday;

pub struct Renderer;

impl Renderer {
    pub fn render_side_a(display: &mut Display5in83, watch: &Watch, temperature: f32, pressure: f32, humidity: f32) {
        //Draw daily info
        let day_of_year = day_of_the_year(watch.date().date, watch.date().month, watch.date().year);
        let a_side_image = ImageManager::a_side(day_of_year - 1); //Image indices start with 0, but days start with 1
        Image::new(&a_side_image, Point::zero()).draw(display).unwrap();

        //Draw layout
        let layout_image = ImageManager::layout();
        Image::new(&layout_image, Point::new(0, 421)).draw(display).unwrap();

        //Render date
        Self::render_date(display, watch);

        //Render moon phase
        let moon_phase = moon_phase(watch.date().date, watch.date().month, watch.date().year);
        let moon_phase_image = ImageManager::moon(moon_phase - 1);
        Image::new(&moon_phase_image, Point::new(112, 486)).draw(display).unwrap();

        //Render air condition
        Self::render_small_digits(display, temperature as u16, Point::new(336, 490), 2);
        Self::render_small_digits(display, (pressure/133.3) as u16, Point::new(336, 520), 3);
        Self::render_small_digits(display, humidity as u16, Point::new(336, 550), 2);
    }

    pub fn render_side_b(display: &mut Display5in83, watch: &Watch) {
        //Draw daily info
        let day_of_year = day_of_the_year(watch.date().date, watch.date().month, watch.date().year);
        let b_side_image = ImageManager::b_side(day_of_year - 1); //Image indices start with 0, but days start with 1
        Image::new(&b_side_image, Point::zero()).draw(display).unwrap();
    }

    fn render_date(display: &mut Display5in83, watch: &Watch) {
        //Draw day of week
        let dow_image = Self::mark_holiday(ImageManager::weekday((watch.date().day - 1) as u8), watch.date()); //Same index shift as for day of year
        Image::new(&dow_image, Point::new(274, 448)).draw(display).unwrap();

        //Draw month
        let month_image = Self::mark_holiday(ImageManager::month((watch.date().month - 1) as u8), watch.date()); //Same index shift as for day of year
        Image::new(&month_image, Point::new(20, 448)).draw(display).unwrap();

        //Draw day
        if watch.date().date < 10 {
            //Simple single digit case
            let day_image = Self::mark_holiday(ImageManager::big_digit((watch.date().date) as u8), watch.date());
            Image::new(&day_image, Point::new(195, 478)).draw(display).unwrap();
        } else {
            //Two digits are slightly more complex
            let left_digit = watch.date().date / 10;
            let right_digit = watch.date().date % 10;
            let left_day_image = Self::mark_holiday(ImageManager::big_digit((left_digit) as u8), watch.date());
            let right_day_image = Self::mark_holiday(ImageManager::big_digit((right_digit) as u8), watch.date());
            Image::new(&left_day_image, Point::new(150, 478)).draw(display).unwrap();
            Image::new(&right_day_image, Point::new(234, 478)).draw(display).unwrap();
        }

        //Draw year
        Self::render_small_digits(display, watch.date().year as u16, Point::new(6, 624), 4);

        //Draw sunrise/sunset
        if let Some(sunrise) = sunrise(watch.date().date, watch.date().month, watch.date().year, watch.lon(), watch.lat()) {
            let hours = sunrise/60;
            let minutes = sunrise-(hours*60);
            Self::render_small_digits(display, hours, Point::new(66, 524), 2);
            Self::render_small_digits(display, minutes, Point::new(110, 524), 2);
        }
        if let Some(sunset) = sunset(watch.date().date, watch.date().month, watch.date().year, watch.lon(), watch.lat()) {
            let hours = sunset/60;
            let minutes = sunset-(hours*60);
            Self::render_small_digits(display, hours, Point::new(66, 550), 2);
            Self::render_small_digits(display, minutes, Point::new(110, 550), 2);
        }
    }

    fn render_small_digits(display: &mut Display5in83, value: u16, position: Point, width: u8) {
        let mut numerator = value;
        let mut current_x = position.x;
        for w in (0..width).rev() {
            let denominator = 10_u32.pow(w as u32);
            let mut  digit = numerator/denominator as u16;
            numerator = numerator - digit * denominator as u16;
            if digit > 9 {
                digit = digit % 10
            }
            let digit_image = ImageManager::small_digit(digit as u8);
            Image::new(&digit_image, Point::new(current_x, position.y)).draw(display).unwrap();
            current_x += 16;
        }
    }

    fn mark_holiday(source: BinImage, date: Date) -> BinImage {
        if date.day == 6 || date.day == 7 || is_holiday(date){
            source.force_chromatic()
        } else {
            source
        }
    }
}