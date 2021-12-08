#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
use alloc_cortex_m::CortexMHeap;
use panic_semihosting as _;

use board::hal;
use cortex_m_rt::entry;
use epd_waveshare::epd5in83b_v2::Display5in83;
use board::hal::delay::Delay;
use board::hal::prelude::*;
use board::hal::rcc::{ClockSecuritySystem, CrystalBypass, MsiFreq};
use crate::watch::Watch;
use epd_waveshare::prelude::*;
use epd_waveshare::prelude::WaveshareThreeColorDisplay;
use board::hal::pwr::{VosRange, WakeUpSource};
use board::shared_delay::SharedDelay;
use crate::renderer::Renderer;

mod watch;
mod gps;
mod bin_image;
mod image_manager;
mod renderer;
mod holiday;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    //Configure heap for image unpacking
    let heap_start = cortex_m_rt::heap_start() as usize;
    let heap_size = 72 * 1024; // 72kb, which should be enough for all the images (55608 bytes max) plus template image (13680 bytes)
    unsafe { ALLOCATOR.init(heap_start, heap_size) } //Heap should be in the SRAM section

    if let Some(mut cp) = cortex_m::Peripherals::take() {
        if let Some(p) = hal::pac::Peripherals::take() {
            // Configure clocks
            let mut rcc = p.RCC.constrain();
            let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
            let mut flash = p.FLASH.constrain();
            let clocks = rcc.cfgr.msi(MsiFreq::RANGE2M).lse(CrystalBypass::Disable, ClockSecuritySystem::Disable).freeze(&mut flash.acr, &mut pwr);

            // Configure lower power mode
            pwr.set_power_range(VosRange::LowPower, &clocks);
            pwr.low_power_run(&clocks);

            // Get access to the interrupts
            let mut exti = p.EXTI;

            //Configure systick as a delay provider
            let systick = cp.SYST;
            let delay = SharedDelay::new(Delay::new(systick, clocks));

            //Get date/time information from the RTC
            let watch = Watch::new(p.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1, &mut exti, p.GPIOD, p.USART2, &mut rcc.ahb2, clocks.clone());

            //Configure board
            let (mut bme280, mut epd_spi, mut epd) = board::init(p.GPIOB, p.I2C1, p.SPI1, &mut rcc.ahb2, &mut rcc.apb1r1, &mut rcc.apb2, clocks.clone(), &delay);

            //Configure display
            //epd.clear_frame(&mut epd_spi, &mut delay.share());

            let mut display = Display5in83::default();
            display.set_rotation(DisplayRotation::Rotate90);

            //Check if we woke up due to the button press and draw B side in that case
            if let Some(WakeUpSource::WKUP1) = pwr.read_wakeup_reason() {
                Renderer::render_side_b(&mut display, &watch);
                epd.update_color_frame(&mut epd_spi, display.bw_buffer(), display.chromatic_buffer()).unwrap();
                epd.display_frame(&mut epd_spi, &mut delay.share()).unwrap();
            } else {
                let air_condition = bme280.measure().unwrap();
                if watch.time().minutes >=0 && watch.time().minutes <= 10 {
                    // Full update in the beginning of the hour
                    Renderer::render_side_a(&mut display, &watch, air_condition.temperature, air_condition.pressure, air_condition.humidity);
                    epd.update_color_frame(&mut epd_spi, display.bw_buffer(), display.chromatic_buffer()).unwrap();
                    epd.display_frame(&mut epd_spi, &mut delay.share()).unwrap();
                } else {
                    //Partial update
                    Renderer::render_air_condition(&mut display,  air_condition.temperature, air_condition.pressure, air_condition.humidity);
                    let mut partial_buf: [u8; 560] = [0; 560];
                    let mut partial_but_index = 0;
                    for y in 336..(336 + 56) {
                        for x in 20..(20 + 10) {
                            partial_buf[partial_but_index] = display.bw_buffer()[y * x + x];
                            partial_but_index += 1;
                        }
                    }
                    epd.update_partial_frame(&mut epd_spi, &partial_buf, 488, 336, 80, 56);
                }
            }

            //Turn off the screen
            epd.sleep(&mut epd_spi, &mut delay.share()).unwrap();

            //Go to the shutdown mode
            pwr.shutdown(&[WakeUpSource::Internal, WakeUpSource::WKUP1], &mut cp.SCB)
        }
    }
    loop {}
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("OOM")
}