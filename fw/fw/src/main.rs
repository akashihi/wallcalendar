#![no_std]
#![no_main]

use panic_semihosting as _;

use board::hal;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_graphics::Drawable;
use embedded_graphics::image::Image;
use embedded_graphics::prelude::{DrawTarget, PixelIteratorExt, Point, Primitive, Size};
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use epd_waveshare::epd5in83b_v2::Display5in83;
use board::hal::delay::Delay;
use board::hal::prelude::*;
use board::hal::rcc::{ClockSecuritySystem, CrystalBypass, MsiFreq};
use crate::watch::Watch;
use epd_waveshare::prelude::*;
use epd_waveshare::prelude::WaveshareDisplay;
use epd_waveshare::prelude::WaveshareThreeColorDisplay;
use board::shared_delay::SharedDelay;
use crate::bin_image::BinImage;

mod watch;
mod gps;
mod bin_image;

#[entry]
fn main() -> ! {
    if let Some(cp) = cortex_m::Peripherals::take() {
        if let Some(p) = hal::pac::Peripherals::take() {
            // Configure clocks
            let mut rcc = p.RCC.constrain();
            let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
            let mut flash = p.FLASH.constrain();
            let clocks = rcc.cfgr.msi(MsiFreq::RANGE4M).lse(CrystalBypass::Disable, ClockSecuritySystem::Disable).freeze(&mut flash.acr, &mut pwr);
            //We need to access PWR after clock freeze to switch into low power run mode
            //unsafe {hal::pac::Peripherals::steal().PWR.cr1.modify(|_, w| w.lpr().set_bit().vos().bits(0b10))};
            let mut exti = p.EXTI;

            //Configure systick as a delay provider
            let systick = cp.SYST;
            let delay = SharedDelay::new(Delay::new(systick, clocks));

            //let watch = Watch::new(p.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1, &mut exti, p.GPIOD, p.USART2, &mut rcc.ahb2, clocks.clone());

            let (mut bme280, mut epd_spi, mut epd) = board::init(p.GPIOB, p.I2C1, p.SPI1, &mut rcc.ahb2, &mut rcc.apb1r1, &mut rcc.apb2, clocks.clone(), &delay);
            //epd.clear_frame(&mut epd_spi, &mut delay.share());
            let mut display = Display5in83::default();
            display.set_rotation(DisplayRotation::Rotate90);

            let a_side_black = include_bytes!("../../../test_images/01-01-a-black.bin");
            let a_side_red = include_bytes!("../../../test_images/01-01-a-red.bin");
            let one = include_bytes!("../../../test_images/1.bin");

            let bin_img = BinImage::from_slice(Size::new(480, 420),a_side_black, Some(a_side_red));
            Image::new(&bin_img, Point::zero()).draw(&mut display);
            let one_img = BinImage::from_slice(Size::new(80, 148),one, None);
            Image::new(&one_img, Point::new(200, 430)).draw(&mut display);

            /*Line::new(Point::new(0, 324), Point::new(480, 324)).into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 2)).draw(&mut display).unwrap();
            Line::new(Point::new(240,0), Point::new(240, 648)).into_styled(PrimitiveStyle::with_stroke(TriColor::Chromatic, 2)).draw(&mut display).unwrap();
            Line::new(Point::new(0,0), Point::new(480, 640)).into_styled(PrimitiveStyle::with_stroke(TriColor::White, 20)).draw(&mut display).unwrap();*/

            epd.update_color_frame(&mut epd_spi, display.bw_buffer(), display.chromatic_buffer()).unwrap();
            epd.display_frame(&mut epd_spi, &mut delay.share()).unwrap();
            loop{
                let air_condition = bme280.measure().unwrap();
                hprintln!("Temperature: {}, Humidity: {}, Pressure: {}", air_condition.temperature, air_condition.humidity, air_condition.pressure);
            }
        }
    }
    loop{}
}