#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::alloc::Layout;
use alloc_cortex_m::CortexMHeap;
use cortex_m::asm::{dsb, wfi};
use panic_semihosting as _;

use board::hal;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use epd_waveshare::epd5in83b_v2::Display5in83;
use board::hal::delay::Delay;
use board::hal::prelude::*;
use board::hal::rcc::{ClockSecuritySystem, CrystalBypass, MsiFreq};
use crate::watch::Watch;
use epd_waveshare::prelude::*;
use epd_waveshare::prelude::WaveshareDisplay;
use epd_waveshare::prelude::WaveshareThreeColorDisplay;
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
    let heap_size = 72*1024; // 72kb, which should be enough for all the images (55608 bytes max) plus template image (13680 bytes)
    unsafe { ALLOCATOR.init(heap_start, heap_size) } //Heap should be in the SRAM section

    if let Some(mut cp) = cortex_m::Peripherals::take() {
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

            let watch = Watch::new(p.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1, &mut exti, p.GPIOD, p.USART2, &mut rcc.ahb2, clocks.clone());

            let (mut bme280, mut epd_spi, mut epd) = board::init(p.GPIOB, p.I2C1, p.SPI1, &mut rcc.ahb2, &mut rcc.apb1r1, &mut rcc.apb2, clocks.clone(), &delay);
            let air_condition = bme280.measure().unwrap();
            //epd.clear_frame(&mut epd_spi, &mut delay.share());
            /*let mut display = Display5in83::default();
            display.set_rotation(DisplayRotation::Rotate90);

            Renderer::render_side_a(&mut display, &watch, air_condition.temperature, air_condition.pressure, air_condition.humidity);
            //Renderer::render_side_b(&mut display, &watch);

            epd.update_color_frame(&mut epd_spi, display.bw_buffer(), display.chromatic_buffer()).unwrap();
            epd.display_frame(&mut epd_spi, &mut delay.share()).unwrap();*/
            //epd.sleep(&mut epd_spi, &mut delay.share()).unwrap();

            //Go to the shutdown mode
            cp.SCB.set_sleepdeep();
            //Configure wakeup sources
            unsafe {
                hal::pac::Peripherals::steal().PWR.cr3.write(|w| w.ewf().set_bit().ewup1().set_bit());
                hal::pac::Peripherals::steal().PWR.scr.write(|w| w.wuf1().set_bit().wuf2().set_bit().wuf3().set_bit().wuf4().set_bit().wuf5().set_bit().sbf().set_bit());
                hal::pac::Peripherals::steal().PWR.cr1.modify(|_, w| w.lpms().bits(0b111))
            }
            unsafe {
                hprintln!("SCB {:#032b}",cp.SCB.scr.read());
                hprintln!("CR1 {:#032b}",hal::pac::Peripherals::steal().PWR.cr1.read().bits());
                hprintln!("CR3 {:#032b}",hal::pac::Peripherals::steal().PWR.cr3.read().bits());
                hprintln!("SR1 {:#032b}",hal::pac::Peripherals::steal().PWR.sr1.read().bits());
            }
            dsb();
            wfi();
            loop{
            }
        }
    }
    loop{}
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    panic!("OOM")
}