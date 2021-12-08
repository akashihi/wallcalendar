use board::hal::hal::serial::Read;
use board::hal::interrupt;
use board::hal::pac::{NVIC, USART2};
use board::hal::serial::{Event, Rx};
use board::{GpsEnPin, GpsUsart};
use core::borrow::{Borrow, BorrowMut};
use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU16, Ordering};
use cortex_m::interrupt as ci;
use cortex_m::interrupt::Mutex;
use nmea::{GpsDate, GpsPosition, GpsTime};

type NmeaBuffer = heapless::String<84>;

static MESSAGES_SEEN: AtomicU16 = AtomicU16::new(0);
static GPS_RX: Mutex<RefCell<Option<Rx<USART2>>>> = Mutex::new(RefCell::new(None));
static RECEIVE_BUFFER: Mutex<RefCell<NmeaBuffer>> =
    Mutex::new(RefCell::new(heapless::String::new()));
static EOL_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

pub(crate) struct Gps {
    en: GpsEnPin,
}

#[allow(non_snake_case)]
#[interrupt]
fn USART2() {
    ci::free(|cs| {
        if let Some(rx) = GPS_RX.borrow(cs).borrow_mut().as_mut() {
            if let Ok(byte) = rx.read() {
                // Read unconditionally to ACK the interrupt
                if byte == 0x0A {
                    return; //We ignore LF
                }
                if !EOL_FLAG.borrow(cs).borrow().get() {
                    //We only modify line if EOL flas is not set, which means processing is etiher not started or already finished
                    if byte == 0x0D {
                        //On CR set end of line flag
                        EOL_FLAG.borrow(cs).borrow_mut().set(true);
                        return; //But do not store the CR
                    }
                    let mut buffer = RECEIVE_BUFFER.borrow(cs).borrow_mut();
                    if buffer.len() > 83 {
                        buffer.clear();
                        // Mark too big string as a message, so we don't spend
                        // too much time reading garbage
                        MESSAGES_SEEN.fetch_add(1, Ordering::Relaxed);
                    }
                    if let Err(_) = buffer.push(byte as char) {
                        //Unable to proceed, let's clear it
                        buffer.clear()
                    }
                }
            }
        }
    })
}

impl Gps {
    pub fn new(mut usart: GpsUsart, mut en: GpsEnPin) -> Self {
        usart.listen(Event::Rxne);
        let (_, rx) = usart.split();
        ci::free(|cs| GPS_RX.borrow(cs).replace(Some(rx)));
        en.set_high(); // Disable GPS receiver
        Gps { en }
    }

    pub fn sync_date_time(&mut self) -> (Option<(GpsDate, GpsTime)>, Option<GpsPosition>) {
        let mut date = None;
        let mut pos = None;
        unsafe {
            NVIC::unmask(interrupt::USART2);
        }
        self.en.set_low(); // Enable GPS receiver
        loop {
            if MESSAGES_SEEN.load(Ordering::Relaxed) > 540 {
                //We should receive one RMC message per second, so after 540 messages(=9 minutes) we time out
                break;
            }
            ci::free(|cs| {
                if EOL_FLAG.borrow(cs).borrow().get() {
                    // Full line is received, parse it
                    let (dt, p) = nmea::parse_nmea_string(&RECEIVE_BUFFER.borrow(cs).borrow());
                    date = dt;
                    pos = p;
                    RECEIVE_BUFFER.borrow(cs).borrow_mut().clear();
                    EOL_FLAG.borrow(cs).borrow_mut().set(false);
                }
            });
            if date.is_some() && pos.is_some() {
                //We got the fix
                break;
            }
            cortex_m::asm::wfi(); //Sleep till next char arrives
        }
        self.en.set_high(); // Disable GPS receiver
        NVIC::mask(interrupt::USART2);
        (date, pos)
    }
}
