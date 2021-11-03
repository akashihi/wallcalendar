#![no_std]
#![no_main]

use panic_semihosting as _;

use board::hal;
use cortex_m_rt::entry;
use board::hal::prelude::*;
use board::hal::rcc::{ClockSecuritySystem, CrystalBypass, MsiFreq};
use crate::gps::Gps;
use crate::watch::Watch;

mod watch;
mod gps;

#[entry]
fn main() -> ! {
    if let Some(cp) = cortex_m::Peripherals::take() {
        if let Some(p) = hal::pac::Peripherals::take() {
            // Configure clocks
            let mut rcc = p.RCC.constrain();
            let mut pwr = p.PWR.constrain(&mut rcc.apb1r1);
            let mut flash = p.FLASH.constrain();
            let clocks = rcc.cfgr.msi(MsiFreq::RANGE16M).lse(CrystalBypass::Disable, ClockSecuritySystem::Disable).freeze(&mut flash.acr, &mut pwr);
            //We need to access PWR after clock freeze to switch into low power run mode
            //unsafe {hal::pac::Peripherals::steal().PWR.cr1.modify(|_, w| w.lpr().set_bit().vos().bits(0b10))};
            let mut exti = p.EXTI;

            let (gps_usart, gps_en) = board::init_uart(p.GPIOD, p.USART2, &mut rcc.ahb2, &mut rcc.apb1r1, clocks.clone());
            let mut gps = Gps::new(gps_usart, gps_en);
            gps.sync_date_time();
            let watch = Watch::new(p.RTC, &mut rcc.apb1r1, &mut rcc.bdcr, &mut pwr.cr1, &mut exti);
        }
    }
    loop{}
}