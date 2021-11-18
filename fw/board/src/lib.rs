#![no_std]

use bme280::BME280;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use epd_waveshare::SPI_MODE;
use epd_waveshare::epd5in83b_v2::Epd5in83;
use epd_waveshare::prelude::WaveshareDisplay;
pub use stm32l4xx_hal as hal;
use stm32l4xx_hal::gpio::{AF4, AF5, AF7, Alternate, Floating, GpioExt, Input, OpenDrain, Output, PB1, PB12, PB13, PB15, PB3, PB4, PB5, PB8, PB9, PD4, PD5, PD6, PushPull};
use stm32l4xx_hal::i2c;
use stm32l4xx_hal::i2c::I2c;
use stm32l4xx_hal::rcc::{AHB2, APB1R1, APB2, Clocks};
use stm32l4xx_hal::serial::{Config, Serial};
use stm32l4xx_hal::time::U32Ext;
use stm32l4xx_hal::pac::{GPIOB, GPIOD, I2C1, SPI1, USART2};
use stm32l4xx_hal::spi::Spi;
use crate::hal::hal::digital::v2::OutputPin;
use crate::shared_delay::{SharedDelay, SharedDelayHandler};

pub mod shared_delay;

pub type RxPin = PD6<Alternate<AF7,PushPull>>;
pub type TxPin = PD5<Alternate<AF7,PushPull>>;
pub type GpsEnPin = PD4<Output<PushPull>>;
pub type GpsUsart = Serial<USART2, (TxPin, RxPin)>;

pub type SCL = PB8<Alternate<AF4,OpenDrain>>;
pub type SDA = PB9<Alternate<AF4,OpenDrain>>;
pub type I2CBus = I2c<I2C1, (SCL, SDA)>;
pub type BmeSensor<'a, D> = BME280<I2CBus, SharedDelayHandler<'a, D>>;

pub type EpdBusy = PB13<Input<Floating>>;
pub type EpdReset = PB12<Output<PushPull>>; //TODO Should be OpenDrain
pub type EpdDC = PB1<Output<PushPull>>;
pub type EpdCs = PB15<Output<PushPull>>;  //TODO Should be OpenDrain
pub type EpdSck = PB3<Alternate<AF5, PushPull>>;
pub type EpdMosi = PB5<Alternate<AF5, PushPull>>;
pub type EpdMiso = PB4<Alternate<AF5, PushPull>>; // Not used by Epd, but required for the SPI
pub type SpiBus = Spi<SPI1, (EpdSck, EpdMiso, EpdMosi)>;
pub type Epd<'a, D> = Epd5in83<SpiBus, EpdCs, EpdBusy, EpdDC, EpdReset, SharedDelayHandler<'a, D>>;

pub fn init<'a, D: DelayMs<u8> + DelayUs<u16>>(gpiob: GPIOB, i2c1: I2C1, spi1: SPI1, ahb2: &mut AHB2, apb1r1: &mut APB1R1, apb2: &mut APB2, clocks: Clocks, delay: &'a SharedDelay<D>) -> (BmeSensor<'a, D>, SpiBus, Epd<'a, D>) {
    let mut port_b = gpiob.split(ahb2);

    // E-Paper SPI interface
    let epd_busy = port_b.pb13.into_floating_input(&mut port_b.moder, &mut port_b.pupdr);
    let mut epd_reset = port_b.pb12.into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
    let epd_dc = port_b.pb1.into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
    let mut epd_cs = port_b.pb15.into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
    let epd_sck = port_b.pb3.into_af5_pushpull(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrl);
    let epd_mosi = port_b.pb5.into_af5_pushpull(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrl);
    let epd_miso = port_b.pb4.into_af5_pushpull(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrl);

    epd_reset.set_high().unwrap_or_default();
    epd_cs.set_high().unwrap_or_default();

    let mut epd_spi = Spi::spi1(
        spi1,
        (epd_sck, epd_miso, epd_mosi),
        SPI_MODE,
        400.khz(),
        clocks.clone(),
        apb2,
    );

    let epd = Epd5in83::new(& mut epd_spi, epd_cs, epd_busy, epd_dc, epd_reset, &mut delay.share()).unwrap();

    let mut scl = port_b.pb8.into_af4_opendrain(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrh);
    scl.internal_pull_up(&mut port_b.pupdr, true);

    let mut sda = port_b.pb9.into_af4_opendrain(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrh);
    sda.internal_pull_up(&mut port_b.pupdr, true);

    let i2c = I2c::i2c1(
        i2c1,
        (scl, sda),
        i2c::Config::new(50.khz(), clocks),
        apb1r1,
    );
    let mut bme280 = BME280::new_primary(i2c, delay.share());
    bme280.init().unwrap();

    (bme280, epd_spi, epd)
}

pub fn init_uart(gpiod: GPIOD, usart2: USART2, ahb2: &mut AHB2, apb1r1: &mut APB1R1, clocks: Clocks) -> (GpsUsart, GpsEnPin) {
    let mut gpio = gpiod.split(ahb2);
    let tx : TxPin = gpio.pd5.into_af7_pushpull(&mut gpio.moder, &mut gpio.otyper, &mut gpio.afrl);
    let rx : RxPin = gpio.pd6.into_af7_pushpull(&mut gpio.moder, &mut gpio.otyper, &mut gpio.afrl);

    let serial = Serial::usart2(usart2, (tx,rx), Config::default().baudrate(9_600.bps()), clocks, apb1r1);

    let mut gps_en_pin:GpsEnPin = gpio.pd4.into_push_pull_output(&mut gpio.moder, &mut gpio.otyper);
    gps_en_pin.set_high().unwrap_or_default(); //Gps power control is active low
    (serial, gps_en_pin)
}
