//! Current date/time information provider
//!
//! * Manages RTC
//! * Syncs RTC with GPS data
//! * Provides wakeup event from the RTC
//! * Provides position information from the GPS

use board::hal::datetime::{Date, Time, U32Ext};
use board::hal::hal::timer::CountDown;
use board::hal::pac::RTC;
use board::hal::pwr::CR1;
use board::hal::rcc::{APB1R1, BDCR};
use board::hal::rtc::{Event, Rtc, RtcConfig};

pub(crate) struct Watch {
    date: Date,
    time: Time,
    lon: f32,
    lat: f32
}

impl Watch {
    pub fn new(rtc_periphery: RTC, apb1r1: &mut APB1R1, bdcr: &mut BDCR, pwrcr1: &mut CR1, exti: &mut board::hal::pac::EXTI) -> Self {
        // Get RTC
        let rtc_config = RtcConfig::default();
        let mut rtc = Rtc::rtc(rtc_periphery, apb1r1, bdcr, pwrcr1, rtc_config);

        // Manage sync flags
        // Sync flags are stored in BKP register 0. We use following 3 flags:
        // * 0xBEEF - No sync needed
        // * 0xC0FE - Sync requested. This is set when we are between 05:00 and 06:00 on Sunday and
        //             current code is 0xBEEF
        // * 0xC0CA - Sync is done. This is set by sync procedure and reset if we are outside of
        //            05:00-06:00 sync window, but code is still 0xC0CA

        let flag_value = rtc.read_backup_register(0);
        if flag_value != 0xBEEF && flag_value != 0xC0CA { //Any other value means that sync is needed
            rtc.write_backup_register(0, 0xC0CA_u32); // Mark as synced
        }
        let (date, time) = rtc.get_date_time();

        //Schedule sync for the next run if needed
        if date.day == 7 && time.hours > 05 && time.hours < 06 {
            if flag_value == 0xBEEF {
                rtc.write_backup_register(0, 0xC0FE_u32); // Mark as synced
            }
        } else {
            //We are outside of sync window, let's reset sync flags
            if flag_value == 0xC0CA {
                rtc.write_backup_register(0, 0xBEEF);
            }
        }

        //Set alarm for next wakeup
        if !rtc.check_interrupt(Event::WakeupTimer, true) {
            //Ok, we didn't woke up because of the RTC WakeUp event, need to set it up for the
            //next wake up
            rtc.listen(exti, Event::WakeupTimer);
            rtc.wakeup_timer().start(10.minutes());
        }

        //Get position, stored in BPK1 (lon) and bkp2 (lon)
        //Coordinates are stored as integers, multiplied by 10^6, which gives good enough resolution
        //for sun/moon calculations
        let lon_i = rtc.read_backup_register(1);
        let lat_i = rtc.read_backup_register(2);

        let lon = lon_i as f32 / 1_000_000.0;
        let lat = lat_i as f32/ 1_000_000.0;

        Watch{date, time, lon, lat}
    }
}