#![no_std]

use bme280::BME280;
pub use stm32l4xx_hal as hal;
use stm32l4xx_hal::gpio::{Alternate, PushPull, AF7, PD6, PD5, PD4, Output, GpioExt, OpenDrain, PB8, PB9, AF4};
use stm32l4xx_hal::i2c;
use stm32l4xx_hal::i2c::I2c;
use stm32l4xx_hal::rcc::{AHB2, APB1R1, Clocks};
use stm32l4xx_hal::serial::{Config, Serial};
use stm32l4xx_hal::time::U32Ext;
use stm32l4xx_hal::pac::{USART2, GPIOD, GPIOB, I2C1};
use crate::hal::hal::digital::v2::OutputPin;

pub type RxPin = PD6<Alternate<AF7,PushPull>>;
pub type TxPin = PD5<Alternate<AF7,PushPull>>;
pub type GpsEnPin = PD4<Output<PushPull>>;
pub type GpsUsart = Serial<USART2, (TxPin, RxPin)>;

pub type SCL = PB8<Alternate<AF4,OpenDrain>>;
pub type SDA = PB9<Alternate<AF4,OpenDrain>>;
type I2CBus = I2c<I2C1, (SCL, SDA)>;
type BmeSensor = BME280<I2CBus, hal::delay::Delay>;

pub fn init(gpiob: GPIOB, i2c1: I2C1, ahb2: &mut AHB2, apb1r1: &mut APB1R1, clocks: Clocks, delay: hal::delay::Delay) -> BmeSensor {
    let mut gpio = gpiob.split(ahb2);
    let mut scl = gpio.pb8.into_af4_opendrain(&mut gpio.moder, &mut gpio.otyper, &mut gpio.afrh);
    scl.internal_pull_up(&mut gpio.pupdr, true);

    let mut sda = gpio.pb9.into_af4_opendrain(&mut gpio.moder, &mut gpio.otyper, &mut gpio.afrh);
    sda.internal_pull_up(&mut gpio.pupdr, true);

    let i2c = I2c::i2c1(
        i2c1,
        (scl, sda),
        i2c::Config::new(50.khz(), clocks),
        apb1r1,
    );
    let mut bme280 = BME280::new_primary(i2c, delay);
    bme280.init().unwrap();
    bme280
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
