#![deny(missing_docs)]
#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

//! Helper module to parse NMEA messages
//! At the moment onl RMC message is supported

use core::ops::{BitXor, Neg};

/// GPS date representation
pub struct GpsDate {
    /// Day of month starting with 1
    pub date: u32,
    /// Month of year starting with 1
    pub month: u32,
    /// Year
    pub year: u32,
}

/// GPS time  representation (UTC)
pub struct GpsTime {
    /// Hour starting with 00
    pub hour: u32,
    /// Minute starting with 0
    pub minute: u32,
    /// Second starting with 0
    pub second: u32
}

/// GPS position representation
pub struct GpsPosition {
    /// Longitude multiplied to 10^6, negative means Western
    pub lon: i32,
    /// Latitude multiplies to 10^6, negative means Southern
    pub lat: i32
}

enum Parts {
    UTC,
    Status,
    Lat,
    LatDir,
    Lon,
    LonDir,
    Speed,
    Track,
    Date,
    MagVar,
    VarDir,
    Mode,
    None
}

impl Parts {
    fn next_part(self) -> Self {
        match self {
            Parts::UTC => {Parts::Status}
            Parts::Status => {Parts::Lat}
            Parts::Lat => {Parts::LatDir}
            Parts::LatDir => {Parts::Lon}
            Parts::Lon => {Parts::LonDir}
            Parts::LonDir => {Parts::Speed}
            Parts::Speed => {Parts::Track}
            Parts::Track => {Parts::Date}
            Parts::Date => {Parts::MagVar}
            Parts::MagVar => {Parts::VarDir}
            Parts::VarDir => {Parts::Mode}
            Parts::Mode => {Parts::None}
            Parts::None => {Parts::None}
        }
    }
}

fn parse_nmea_date(nmea: &str) -> Option<GpsDate> {
    let day = nmea.get(0..2).and_then(|s| u32::from_str_radix(s, 10).ok());
    let month = nmea.get(2..4).and_then(|s| u32::from_str_radix(s, 10).ok());
    let year = nmea.get(4..6).and_then(|s| u32::from_str_radix(s, 10).ok()).map(|y| y + 2000);
    match (day, month, year) {
        (Some(d), Some(m), Some(y)) => Some(GpsDate { date: d, month: m, year: y }),
        _ => None
    }
}

fn parse_nmea_time(nmea: &str) -> Option<GpsTime> {
    let hour = nmea.get(0..2).and_then(|s| u32::from_str_radix(s, 10).ok());
    let minute = nmea.get(2..4).and_then(|s| u32::from_str_radix(s, 10).ok());
    let second = nmea.get(4..6).and_then(|s| u32::from_str_radix(s, 10).ok());
    match (hour, minute, second) {
        (Some(h), Some(m), Some(s)) => Some(GpsTime { hour: h, minute: m, second: s }),
        _ => None
    }
}

fn parse_nmea_coords(nmea: &str) -> Option<i32> {
    nmea.parse::<f32>().ok().map(|v| v*10000.0).map(|v| v as i32)
}

fn calculate_checksum(message: &str) -> u8 {
    let mut checksum = 0x00;
    for byte in message.as_bytes() {
        checksum = checksum.bitxor(byte);
    }
    checksum
}

fn check_checksum(nmea: &str) -> bool {
    nmea.get(1..).map(|s| s.split('*'))
        .map(|mut parts| (parts.next(), parts.next()))
        .map(|(message, checksum)| (message, checksum.and_then(|cs| u8::from_str_radix(cs, 16).ok())))
        .and_then(|(message, checksum)| message.zip(checksum))
        .map(|(message, checksum)| (calculate_checksum(message), checksum))
        .map(|(actual_checksum, expected_checksum)| actual_checksum == expected_checksum)
        .unwrap_or(false)
}

/// Takes a NMEA message, parses it and produces time and position information
///
/// May return double None is message is non-RMC or no data in RMC message
/// Returns Date only is not fix available yet
/// Returns both Data nd Position if stable fix is reported in RMC message
pub fn parse_nmea_string(nmea: &str) -> (Option<(GpsDate, GpsTime)>, Option<GpsPosition>) {
    let mut date = None;
    let mut time = None;
    let mut lon = None;
    let mut lat = None;
    if let Some(message) = nmea.get(3..6) {
        if message == "RMC" {
            if check_checksum(nmea) {
                let mut current_part = Parts::UTC;
                for part in nmea[7..].split(',') {
                    match current_part {
                        Parts::UTC => time = parse_nmea_time(part),
                        Parts::Date => date = parse_nmea_date(part),
                        Parts::Lon => lon = parse_nmea_coords(part),
                        Parts::Lat => lat = parse_nmea_coords(part),
                        Parts::LonDir => if part == "W" { lon = lon.map(|v| v.neg())},
                        Parts::LatDir => if part == "S" { lat = lat.map(|v| v.neg())},
                        Parts::Mode => { //This is our exit condition
                            let position = match (lon, lat) {
                                (Some(lo), Some(la)) => Some(GpsPosition{lon: lo, lat: la}),
                                _ => None
                            };
                            return (date.zip(time), position)
                        },
                        _ => {/* ignore that part */}
                    }
                    current_part = current_part.next_part();
                }
            }
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use crate::{parse_nmea_coords, parse_nmea_date, parse_nmea_string};

    #[test]
    fn date_too_short() {
        assert!(parse_nmea_date("0311").is_none())
    }
    #[test]
    fn date_non_numeric() {
        assert!(parse_nmea_date("0311aa").is_none())
    }
    #[test]
    fn date_correct() {
        let date = parse_nmea_date("031121");
        assert!(date.is_some());
        let dt = date.unwrap();
        assert_eq!(dt.date, 03);
        assert_eq!(dt.month, 11);
        assert_eq!(dt.year, 2021);
    }

    #[test]
    fn parse_coords_non_float() {
        assert!(parse_nmea_coords("3.14aa").is_none());
    }
    #[test]
    fn parse_coords() {
        let value = parse_nmea_coords("6005.84256");
        assert!(value.is_some());
        let v = value.unwrap();
        assert_eq!(v/10, 6005842)
    }

    #[test]
    fn skip_non_rmc_message() {
        let (date, position) = parse_nmea_string("$GPGSV,3,1,12,01,15,170,20,02,08,326,18,03,63,126,22,04,66,205,*7D");
        assert!(date.is_none());
        assert!(position.is_none());
    }

    #[test]
    fn empty_rmc_message_is_ignored() {
        let (date, position) = parse_nmea_string("$GPRMC,,V,,,,,,,,,,N*53");
        assert!(date.is_none());
        assert!(position.is_none());
    }

    #[test]
    fn time_only_rmc_message_is_ignored() {
        let (date, position) = parse_nmea_string("$GPRMC,092618.51,V,,,,,,,,,,N*7D");
        assert!(date.is_none());
        assert!(position.is_none());
    }

    #[test]
    fn no_time_without_date() {
        let (date, position) = parse_nmea_string("$GPRMC,092618.51,V,,,,,,,,,,N*7D");
        assert!(date.is_none());
        assert!(position.is_none());
    }

    #[test]
    fn date_only_rmc_message_is_parsed() {
        let (date, position) = parse_nmea_string("$GPRMC,092623.00,V,,,,,,,031121,,,N*71");
        assert!(date.is_some());
        assert!(position.is_none());

        let (d,t) = date.unwrap();
        assert_eq!(d.date, 03);
        assert_eq!(d.month, 11);
        assert_eq!(d.year, 2021);

        assert_eq!(t.hour, 09);
        assert_eq!(t.minute, 26);
        assert_eq!(t.second, 23);
    }

    #[test]
    fn full_rmc_message_is_parsed() {
        let (date, position) = parse_nmea_string("$GPRMC,093052.00,A,6005.84256,N,02414.01597,E,1.055,,031121,,,A*7B");
        assert!(date.is_some());
        assert!(position.is_some());

        let (d, t) = date.unwrap();
        assert_eq!(d.date, 03);
        assert_eq!(d.month, 11);
        assert_eq!(d.year, 2021);

        assert_eq!(t.hour, 09);
        assert_eq!(t.minute, 30);
        assert_eq!(t.second, 52);

        let pos = position.unwrap();
        assert_eq!(pos.lon/10, 2414015);
        assert_eq!(pos.lat/10, 6005842);
    }

    #[test]
    fn wrong_checksum_ignored() {
        let (date, position) = parse_nmea_string("$GPRMC,093052.00,A,6005.84256,N,02414.01597,E,1.055,,031121,,,A*B7");
        assert!(date.is_none());
        assert!(position.is_none());
    }

}