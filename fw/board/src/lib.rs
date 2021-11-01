#![no_std]

/*use crate::hal::gpio::gpioa::{PA10, PA9};
use crate::hal::gpio::gpiob::{PB14, PB15};
use crate::hal::gpio::gpioe::PEx;
use crate::hal::gpio::GpioExt;
use core::ops;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::gpioa::{PA0, PA1, PA15, PA2, PA3, PA4, PA5, PA6, PA7};
use hal::gpio::gpiob::{PB0, PB1, PB10, PB11, PB3, PB4, PB5, PB6, PB7, PB8, PB9};
use hal::gpio::gpioc::{PC13, PC14, PC15};
use hal::gpio::{Floating, Input, Output, PushPull};
use hal::pac::{GPIOA, GPIOB, GPIOC};*/
pub use stm32l4xx_hal as hal;


/*pub type LdGreen = PB0<Output<PushPull>>;
pub type LdRed = PB1<Output<PushPull>>;
pub type LdYellow = PB10<Output<PushPull>>;
pub type LdBlue = PB11<Output<PushPull>>;

pub type LdBtnGreen = PA4<Output<PushPull>>;
pub type LdBtnRed = PA5<Output<PushPull>>;
pub type LdBtnYellow = PA6<Output<PushPull>>;
pub type LdBtnBlue = PA7<Output<PushPull>>;

pub type BtnGreen = PB5<Input<Floating>>;
pub type BtnRed = PA1<Input<Floating>>;
pub type BtnYellow = PA2<Input<Floating>>;
pub type BtnBlue = PA3<Input<Floating>>;

pub type BtnMode = PC13<Input<Floating>>;
pub type BtnRight = PC14<Input<Floating>>;
pub type BtnDown = PC15<Input<Floating>>;
pub type BtnSet = PA0<Input<Floating>>;

pub type Hd4BitBus = FourBitBus<
    PB4<Output<PushPull>>,
    PB3<Output<PushPull>>,
    PB9<Output<PushPull>>,
    PB8<Output<PushPull>>,
    PB7<Output<PushPull>>,
    PB6<Output<PushPull>>,
>;
pub type Lcd = HD44780<Hd4BitBus>;

pub type SsClk = PB15<Output<PushPull>>;
pub type SsReset = PA10<Output<PushPull>>;
pub type SsOe = PA9<Output<PushPull>>;
pub type SsData = PB14<Output<PushPull>>;

pub type LedScreen = SevenSegment<SsReset, SsOe, SsClk, SsData>;

pub type Speaker = Beeper<PA15<Output<PushPull>>>;*/

/*pub fn init<D: DelayUs<u16> + DelayMs<u8>>(
    gpioa: GPIOA,
    gpiob: GPIOB,
    gpioc: GPIOC,
    apb2: &mut hal::rcc::APB2,
    afio: &mut hal::afio::Parts,
    delay: &mut D,
) -> (Leds, LedScreen, Lcd, Speaker, ButtonSet) {
    let mut port_a = gpioa.split(apb2);
    let mut port_b = gpiob.split(apb2);
    let mut port_c = gpioc.split(apb2);

    let ld_green = port_b.pb0.into_push_pull_output(&mut port_b.crl);
    let ld_red = port_b.pb1.into_push_pull_output(&mut port_b.crl);
    let ld_yellow = port_b.pb10.into_push_pull_output(&mut port_b.crh);
    let ld_blue = port_b.pb11.into_push_pull_output(&mut port_b.crh);
    let ld_btn_green = port_a.pa4.into_push_pull_output(&mut port_a.crl);
    let ld_btn_red = port_a.pa5.into_push_pull_output(&mut port_a.crl);
    let ld_btn_yellow = port_a.pa6.into_push_pull_output(&mut port_a.crl);
    let ld_btn_blue = port_a.pa7.into_push_pull_output(&mut port_a.crl);

    let leds = Leds {
        leds: [
            ld_green.into(),
            ld_red.into(),
            ld_yellow.into(),
            ld_blue.into(),
            ld_btn_green.into(),
            ld_btn_red.into(),
            ld_btn_yellow.into(),
            ld_btn_blue.into(),
        ],
    };

    let ss_clk = port_b.pb15.into_push_pull_output(&mut port_b.crh);
    let ss_reset = port_a.pa10.into_push_pull_output(&mut port_a.crh);
    let ss_oe = port_a.pa9.into_push_pull_output(&mut port_a.crh);
    let ss_data = port_b.pb14.into_push_pull_output(&mut port_b.crh);
    let screen = LedScreen::new(ss_reset, ss_oe, ss_clk, ss_data);

    let (pa15, pb3, pb4) = afio.mapr.disable_jtag(port_a.pa15, port_b.pb3, port_b.pb4);
    let pin_rs = pb4.into_push_pull_output(&mut port_b.crl);
    let pin_e = pb3.into_push_pull_output(&mut port_b.crl);
    let pin_d4 = port_b.pb9.into_push_pull_output(&mut port_b.crh);
    let pin_d5 = port_b.pb8.into_push_pull_output(&mut port_b.crh);
    let pin_d6 = port_b.pb7.into_push_pull_output(&mut port_b.crl);
    let pin_d7 = port_b.pb6.into_push_pull_output(&mut port_b.crl);

    let lcd = HD44780::new_4bit(pin_rs, pin_e, pin_d4, pin_d5, pin_d6, pin_d7, delay)
        .and_then(|mut l| {
            l.reset(delay)?;
            Result::Ok(l)
        })
        .and_then(|mut l| {
            l.clear(delay)?;
            Result::Ok(l)
        })
        .and_then(|mut l| {
            l.set_cursor_visibility(Cursor::Invisible, delay)?;
            Result::Ok(l)
        })
        .unwrap();

    let beeper_pin = pa15.into_push_pull_output(&mut port_a.crh);
    let speaker = Beeper::new(beeper_pin);

    let btn_green = port_b.pb5.into_floating_input(&mut port_b.crl);
    let btn_red = port_a.pa1.into_floating_input(&mut port_a.crl);
    let btn_yellow = port_a.pa2.into_floating_input(&mut port_a.crl);
    let btn_blue = port_a.pa3.into_floating_input(&mut port_a.crl);

    let btn_mode = port_c.pc13.into_floating_input(&mut port_c.crh);
    let btn_right = port_c.pc14.into_floating_input(&mut port_c.crh);
    let btn_down = port_c.pc15.into_floating_input(&mut port_c.crh);
    let btn_set = port_a.pa0.into_floating_input(&mut port_a.crl);

    let buttons = ButtonSet {
        buttons: [
            Button::new(btn_mode.downgrade()),
            Button::new(btn_right.downgrade()),
            Button::new(btn_down.downgrade()),
            Button::new(btn_set.downgrade()),
            Button::new(btn_green.downgrade()),
            Button::new(btn_red.downgrade()),
            Button::new(btn_yellow.downgrade()),
            Button::new(btn_blue.downgrade()),
        ],
    };

    (leds, screen, lcd, speaker, buttons)
}

pub enum LedColors {
    Green,
    Red,
    Yellow,
    Blue,
    ButtonGreen,
    ButtonRed,
    ButtonYellow,
    ButtonBlue,
}

pub struct Led {
    pin: PEx<Output<PushPull>>,
}

macro_rules! ctor {
    ($($ldx:ty),+) => {
        $(
            impl From<$ldx> for Led {
                fn from(pin: $ldx) -> Self {
                    Led {
                        pin: pin.downgrade(),
                    }
                }
            }
        )+
    }
}

ctor!(
    LdGreen,
    LdRed,
    LdYellow,
    LdBlue,
    LdBtnGreen,
    LdBtnRed,
    LdBtnYellow,
    LdBtnBlue
);

impl ops::Deref for Leds {
    type Target = [Led];

    fn deref(&self) -> &[Led] {
        &self.leds
    }
}

impl ops::DerefMut for Leds {
    fn deref_mut(&mut self) -> &mut [Led] {
        &mut self.leds
    }
}

impl ops::Index<usize> for Leds {
    type Output = Led;

    fn index(&self, i: usize) -> &Led {
        &self.leds[i]
    }
}

impl ops::Index<LedColors> for Leds {
    type Output = Led;

    fn index(&self, d: LedColors) -> &Led {
        &self.leds[d as usize]
    }
}

impl ops::IndexMut<usize> for Leds {
    fn index_mut(&mut self, i: usize) -> &mut Led {
        &mut self.leds[i]
    }
}

impl ops::IndexMut<LedColors> for Leds {
    fn index_mut(&mut self, d: LedColors) -> &mut Led {
        &mut self.leds[d as usize]
    }
}

impl Led {
    pub fn on(&mut self) {
        self.pin.set_high().unwrap_or_default()
    }

    pub fn off(&mut self) {
        self.pin.set_low().unwrap_or_default()
    }
}

pub struct Leds {
    leds: [Led; 8],
}

impl Leds {
    pub fn all_on(&mut self) {
        for led in self.leds.iter_mut() {
            led.on()
        }
    }

    pub fn all_off(&mut self) {
        for led in self.leds.iter_mut() {
            led.off()
        }
    }
}

pub struct Beeper<PIN: OutputPin> {
    pin: PIN,
}

impl<PIN: OutputPin> Beeper<PIN> {
    pub fn new(mut pin: PIN) -> Self {
        pin.set_high().unwrap_or_default(); //Mute the beeper
        Beeper { pin }
    }

    pub fn on(&mut self) {
        self.pin.set_low().unwrap_or_default()
    }

    pub fn off(&mut self) {
        self.pin.set_high().unwrap_or_default()
    }
}*/
