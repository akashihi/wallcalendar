use celestial::{day_of_the_year, moon_phase};
use cortex_m_semihosting::hprintln;
use embedded_graphics::image::Image;
use embedded_graphics::prelude::*;
use epd_waveshare::epd5in83b_v2::Display5in83;
use crate::{ImageManager, Watch};

pub struct Renderer;

impl Renderer {
    pub fn render_side_a(display: &mut Display5in83, watch: &Watch) {
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
    }

    fn render_date(display: &mut Display5in83, watch: &Watch) {
        //Draw day of week
        let dow_image = ImageManager::weekday((watch.date().day - 1) as u8); //Same index shift as for day of year
        Image::new(&dow_image, Point::new(274, 448)).draw(display).unwrap();

        //Draw month
        let month_image = ImageManager::month((watch.date().month - 1) as u8); //Same index shift as for day of year
        Image::new(&month_image, Point::new(20, 448)).draw(display).unwrap();

        //Draw day
        if watch.date().date < 10 {
            //Simple single digit case
            let day_image = ImageManager::big_digit((watch.date().date) as u8);
            Image::new(&day_image, Point::new(195, 478)).draw(display).unwrap();
        } else {
            //Two digits are slightly more complex
            let left_digit = watch.date().date / 10;
            let right_digit = watch.date().date % 10;
            let left_day_image = ImageManager::big_digit((left_digit) as u8);
            let right_day_image = ImageManager::big_digit((right_digit) as u8);
            Image::new(&left_day_image, Point::new(150, 478)).draw(display).unwrap();
            Image::new(&right_day_image, Point::new(234, 478)).draw(display).unwrap();
        }

        //Draw year
        let year_thousands = watch.date().year / 1000;
        let year_hundreds = (watch.date().year % 1000) / 100;
        let year_tens = (watch.date().year % 100) / 10;
        let year_ones = watch.date().year % 10;
        let thousands_year_image = ImageManager::small_digit((year_thousands) as u8);
        let hundreds_year_image = ImageManager::small_digit((year_hundreds) as u8);
        let tens_year_image = ImageManager::small_digit((year_tens) as u8);
        let ones_year_image = ImageManager::small_digit((year_ones) as u8);
        Image::new(&thousands_year_image, Point::new(6, 624)).draw(display).unwrap();
        Image::new(&hundreds_year_image, Point::new(24, 624)).draw(display).unwrap();
        Image::new(&tens_year_image, Point::new(40, 624)).draw(display).unwrap();
        Image::new(&ones_year_image, Point::new(56, 624)).draw(display).unwrap();
    }
}