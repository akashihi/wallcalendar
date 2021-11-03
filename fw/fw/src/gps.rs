use core::borrow::{Borrow, BorrowMut};
use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU16, Ordering};
use cortex_m::interrupt::Mutex;
use board::{GpsEnPin, GpsUsart};
use board::hal::serial::{Event, Rx};
use board::hal::interrupt;
use board::hal::pac::{NVIC, USART2};
use cortex_m::interrupt as ci;
use board::hal::hal::serial::Read;
use cortex_m_semihosting::hprintln;
use cortex_m::iprint;
use board::hal::hal::digital::v2::OutputPin;

type NmeaBuffer = heapless::String<84>;

static MESSAGES_SEEN: AtomicU16 = AtomicU16::new(0);
static GPS_RX: Mutex<RefCell<Option<Rx<USART2>>>> = Mutex::new(RefCell::new(None));
static RECEIVE_BUFFER: Mutex<RefCell<NmeaBuffer>> = Mutex::new(RefCell::new(heapless::String::new()));
static EOL_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

pub(crate) struct Gps {
    en: GpsEnPin
}

#[allow(non_snake_case)]
#[interrupt]
fn USART2() {
    ci::free(|cs| {
        if let Some(rx) = GPS_RX.borrow(cs).borrow_mut().as_mut() {
            if let Ok(byte) = rx.read() { // Read unconditionally to ACK the interrupt
                if byte == 0x0A {
                    return; //We ignore LF
                }
                if !EOL_FLAG.borrow(cs).borrow().get() { //We only modify line if EOL flas is not set, which means processing is etiher not started or already finished
                    if byte == 0x0D { //On CR set end of line flag
                        EOL_FLAG.borrow(cs).borrow_mut().set(true);
                        return; //But do not store the CR
                    }
                    let mut buffer = RECEIVE_BUFFER.borrow(cs).borrow_mut();
                    if buffer.len()>83 {
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

fn parse_nmea_string(nmea: &str) -> Option<()> {
    if let Some(message) = nmea.get(3..6) {
        if message == "RMC" {
            MESSAGES_SEEN.fetch_add(1, Ordering::Relaxed);
            hprintln!("{}", nmea);
        }
    }
    None
}

impl Gps {
    pub fn new(mut usart: GpsUsart, mut en: GpsEnPin) -> Self {
        usart.listen(Event::Rxne);
        let (_, rx) = usart.split();
        ci::free(|cs| GPS_RX.borrow(cs).replace(Some(rx)));
        unsafe { NVIC::unmask(interrupt::USART2); }
        en.set_low().unwrap_or_default(); // Enable GPS receiver
        Gps {en}
    }

    pub fn sync_date_time(&mut self) {
        loop {
            if MESSAGES_SEEN.load(Ordering::Relaxed) > 540 { //We should receive one RMC message per second, so after 540 messages(=9 minutes) we time out
                break;
            }
            ci::free(|cs| {
                if EOL_FLAG.borrow(cs).borrow().get() {
                    // Full line is received, parse it
                    parse_nmea_string(&RECEIVE_BUFFER.borrow(cs).borrow());
                    RECEIVE_BUFFER.borrow(cs).borrow_mut().clear();
                    EOL_FLAG.borrow(cs).borrow_mut().set(false);
                }
            });
            cortex_m::asm::wfi(); //Sleep till next char arrives
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gps::parse_nmea_string;

    #[test]
    fn non_rmc_message_skipped() {
        assert!(parse_nmea_string("$GPGSV,3,1,12,01,15,170,20,02,08,326,18,03,63,126,22,04,66,205,*7D").is_none());
    }
}