#![no_std]

use crate::hal::rcc::Enable;
use crate::shared_delay::{SharedDelay, SharedDelayHandler};
use bme280::BME280;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use epd_waveshare::epd5in83b_v2::Epd5in83;
use epd_waveshare::prelude::WaveshareDisplay;
use epd_waveshare::SPI_MODE;
pub use stm32l4xx_hal as hal;
use stm32l4xx_hal::gpio::{
    Alternate, Floating, GpioExt, Input, OpenDrain, Output, Pin, PushPull, H8, L8,
};
use stm32l4xx_hal::i2c;
use stm32l4xx_hal::i2c::I2c;
use stm32l4xx_hal::pac::{GPIOA, GPIOB, GPIOD, GPIOE, I2C1, QUADSPI, SPI1, USART1, USART2};
use stm32l4xx_hal::rcc::{Clocks, AHB2, AHB3, APB1R1, APB2};
use stm32l4xx_hal::serial::{Config, Serial};
use stm32l4xx_hal::spi::Spi;
use stm32l4xx_hal::time::U32Ext;

pub mod shared_delay;

pub type QspiReset = Pin<Output<PushPull>, L8, 'A', 3>;
pub type QspiCs = Pin<Alternate<PushPull, 10>, L8, 'A', 2>;
pub type QspiClk = Pin<Alternate<PushPull, 10>, H8, 'B', 10>;
pub type QspiIO3 = Pin<Alternate<PushPull, 10>, H8, 'E', 15>;
pub type QspiIO2 = Pin<Alternate<PushPull, 10>, H8, 'E', 14>;
pub type QspiIO1 = Pin<Alternate<PushPull, 10>, L8, 'B', 0>;
pub type QspiIO0 = Pin<Alternate<PushPull, 10>, H8, 'E', 12>;

pub type ExtRxPin = Pin<Alternate<PushPull, 7>, H8, 'A', 10>;
pub type ExtTxPin = Pin<Alternate<PushPull, 7>, H8, 'A', 9>;
pub type ExtUsart = Serial<USART1, (ExtTxPin, ExtRxPin)>;

pub type RxPin = Pin<Alternate<PushPull, 7>, L8, 'D', 6>;
pub type TxPin = Pin<Alternate<PushPull, 7>, L8, 'D', 5>;
pub type GpsEnPin = Pin<Output<PushPull>, L8, 'D', 4>;
pub type GpsUsart = Serial<USART2, (TxPin, RxPin)>;

pub type SCL = Pin<Alternate<OpenDrain, 4>, H8, 'B', 8>;
pub type SDA = Pin<Alternate<OpenDrain, 4>, H8, 'B', 9>;
pub type I2CBus = I2c<I2C1, (SCL, SDA)>;
pub type BmeSensor<'a, D> = BME280<I2CBus, SharedDelayHandler<'a, D>>;

pub type EpdBusy = Pin<Input<Floating>, H8, 'B', 13>;
pub type EpdReset = Pin<Output<PushPull>, H8, 'B', 12>; //TODO Should be OpenDrain
pub type EpdDC = Pin<Output<PushPull>, L8, 'B', 1>;
pub type EpdCs = Pin<Output<PushPull>, H8, 'B', 15>; //TODO Should be OpenDrain
pub type EpdSck = Pin<Alternate<PushPull, 5>, L8, 'B', 3>;
pub type EpdMosi = Pin<Alternate<PushPull, 5>, L8, 'B', 5>;
pub type EpdMiso = Pin<Alternate<PushPull, 5>, L8, 'B', 4>; // Not used by Epd, but required for the SPI
pub type SpiBus = Spi<SPI1, (EpdSck, EpdMiso, EpdMosi)>;
pub type Epd<'a, D> = Epd5in83<SpiBus, EpdCs, EpdBusy, EpdDC, EpdReset, SharedDelayHandler<'a, D>>;

pub fn init<'a, D: DelayMs<u8> + DelayUs<u16>>(
    gpioa: GPIOA,
    gpiob: GPIOB,
    gpioe: GPIOE,
    quadspi: QUADSPI,
    i2c1: I2C1,
    spi1: SPI1,
    ahb2: &mut AHB2,
    ahb3: &mut AHB3,
    apb1r1: &mut APB1R1,
    apb2: &mut APB2,
    clocks: Clocks,
    delay: &'a SharedDelay<D>,
) -> (BmeSensor<'a, D>, SpiBus, Epd<'a, D>) {
    let mut port_a = gpioa.split(ahb2);
    let mut port_b = gpiob.split(ahb2);
    let mut port_e = gpioe.split(ahb2);

    //Wake-up button
    port_a
        .pa0
        .into_floating_input(&mut port_a.moder, &mut port_a.pupdr); //We only need to configure it for wake-up, no actual access is needed

    //QSPI
    let mut qspi_reset = port_a.pa3.into_push_pull_output(&mut port_a.moder, &mut port_a.otyper);
    let _qspi_cs: QspiCs =
        port_a
            .pa2
            .into_alternate(&mut port_a.moder, &mut port_a.otyper, &mut port_a.afrl);
    let _qspi_clk: QspiClk =
        port_b
            .pb10
            .into_alternate(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrh);
    let _qspi_io3: QspiIO3 =
        port_e
            .pe15
            .into_alternate(&mut port_e.moder, &mut port_e.otyper, &mut port_e.afrh);
    let _qspi_io2: QspiIO2 =
        port_e
            .pe14
            .into_alternate(&mut port_e.moder, &mut port_e.otyper, &mut port_e.afrh);
    let _qspi_io1: QspiIO1 =
        port_b
            .pb0
            .into_alternate(&mut port_b.moder, &mut port_b.otyper, &mut port_b.afrl);
    let _qspi_io0: QspiIO0 =
        port_e
            .pe12
            .into_alternate(&mut port_e.moder, &mut port_e.otyper, &mut port_e.afrh); //IO0

    /* --- HAL implementation is buggy, have to configure manually --- */
    //let qspi_config = QspiConfig::default().flash_size(23).address_size(AddressSize::Addr32Bit).qpi_mode(true); //We expect 16MB flash
    //let qspi = Qspi::new(quadspi, (qspi_clk, qspi_cs, qspi_io0, qspi_io1, qspi_io2, qspi_io3), ahb3, qspi_config);
    qspi_reset.set_high(); //Activate flash before using it
    QUADSPI::enable(ahb3);
    unsafe {
        quadspi.ccr.modify(|_, w| {
            w.instruction()
                .bits(0xEE) //DDR Quad IO wth 4 byte address sent 4 bits at a time
                .ddrm()
                .set_bit() //Double data rate
                .dhhc()
                .set_bit() //1/4 pulse sampling
                .fmode()
                .bits(0b11) //Memory mapped IO
                .dmode()
                .bits(0b11) //Data on four lines
                .abmode()
                .bits(0b00) //No alternate bytes
                .adsize()
                .bits(0b11) //4 byte address
                .admode()
                .bits(0b11) //Address on 4 lines
                .imode()
                .bits(0b01)
        }); //Instruction in a IO0
        quadspi.dcr.modify(|_, w| {
            w.fsize()
                .bits(23) //16MB flash
                .ckmode()
                .set_bit()
        }) //Mode 3
    }
    quadspi.cr.modify(|_, w| w.en().set_bit());

    // E-Paper SPI interface
    let epd_busy = port_b
        .pb13
        .into_floating_input(&mut port_b.moder, &mut port_b.pupdr);
    let mut epd_reset = port_b
        .pb12
        .into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
    let epd_dc = port_b
        .pb1
        .into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
    let mut epd_cs = port_b
        .pb15
        .into_push_pull_output(&mut port_b.moder, &mut port_b.otyper);
    let epd_sck = port_b.pb3.into_alternate_push_pull(
        &mut port_b.moder,
        &mut port_b.otyper,
        &mut port_b.afrl,
    );
    let epd_mosi = port_b.pb5.into_alternate_push_pull(
        &mut port_b.moder,
        &mut port_b.otyper,
        &mut port_b.afrl,
    );
    let epd_miso = port_b.pb4.into_alternate_push_pull(
        &mut port_b.moder,
        &mut port_b.otyper,
        &mut port_b.afrl,
    );

    epd_reset.set_high();
    epd_cs.set_high();

    let mut epd_spi = Spi::spi1(
        spi1,
        (epd_sck, epd_miso, epd_mosi),
        SPI_MODE,
        400.khz(),
        clocks,
        apb2,
    );

    let epd = Epd5in83::new(
        &mut epd_spi,
        epd_cs,
        epd_busy,
        epd_dc,
        epd_reset,
        &mut delay.share(),
    )
    .unwrap();

    let mut scl = port_b.pb8.into_alternate_open_drain(
        &mut port_b.moder,
        &mut port_b.otyper,
        &mut port_b.afrh,
    );
    scl.internal_pull_up(&mut port_b.pupdr, true);

    let mut sda = port_b.pb9.into_alternate_open_drain(
        &mut port_b.moder,
        &mut port_b.otyper,
        &mut port_b.afrh,
    );
    sda.internal_pull_up(&mut port_b.pupdr, true);

    let i2c = I2c::i2c1(i2c1, (scl, sda), i2c::Config::new(50.khz(), clocks), apb1r1);
    let mut bme280 = BME280::new_primary(i2c, delay.share());
    bme280.init().unwrap();

    (bme280, epd_spi, epd)
}

pub fn init_uart(
    gpiod: GPIOD,
    usart2: USART2,
    ahb2: &mut AHB2,
    apb1r1: &mut APB1R1,
    clocks: Clocks,
) -> (GpsUsart, GpsEnPin) {
    let mut gpio = gpiod.split(ahb2);
    let tx: TxPin =
        gpio.pd5
            .into_alternate_push_pull(&mut gpio.moder, &mut gpio.otyper, &mut gpio.afrl);
    let rx: RxPin =
        gpio.pd6
            .into_alternate_push_pull(&mut gpio.moder, &mut gpio.otyper, &mut gpio.afrl);

    let serial = Serial::usart2(
        usart2,
        (tx, rx),
        Config::default().baudrate(9_600.bps()),
        clocks,
        apb1r1,
    );

    let mut gps_en_pin: GpsEnPin = gpio
        .pd4
        .into_push_pull_output(&mut gpio.moder, &mut gpio.otyper);
    gps_en_pin.set_high(); //Gps power control is active low
    (serial, gps_en_pin)
}
