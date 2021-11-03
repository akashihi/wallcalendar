#![deny(missing_docs)]
#![deny(unsafe_code)]

//! Sunrise/Sunset calculation

use core::f32::consts::PI;
use micromath::F32Ext;

const ZENITH: f32 = (PI/180.0)*96.0; // Civil twilight

/// Calculates sunrise for a location and date
/// Sunrise in UTC minutes
pub fn sunrise(day: u32, month: u32, year: u32, lon: f32, lat: f32) -> Option<u16> {
    let year_day = day_of_the_year(day, month, year);
    let lng_hour = lon/15.0; //longitude hour
    let t = year_day as f32 + ((6.0 - lng_hour) / 24.0); //for sunrise
    let m = (0.9856 * t) - 3.289; //sun's mean anomaly
    let l = m + (1.916*f32::sin(deg_to_rad(m))) + (0.020*f32::sin(2.0*deg_to_rad(m))) + 282.634;
    let l_adjusted = if l < 0.0 { l + 360.0 } else if l > 360.0 { l - 360.0} else { l };
    let ra = rad_to_deg(f32::atan(0.91764 * f32::tan(deg_to_rad(l_adjusted))));
    let l_quadrant  = f32::floor( l_adjusted/90.0) * 90.0;
    let ra_quadrant = f32::floor(ra/90.0) * 90.0;
    let ra_adjusted = (ra + (l_quadrant - ra_quadrant))/15.0;
    let sin_dec = 0.39782 * f32::sin(deg_to_rad(l_adjusted));
    let cos_dec = f32::cos(f32::asin(sin_dec));
    let cos_h = (f32::cos(ZENITH) - (sin_dec * f32::sin(deg_to_rad(lat)))) / (cos_dec * f32::cos(deg_to_rad(lat)));

    if cos_h > 1.0 || cos_h < -1.0 { None } else {
        let h = (360.0-rad_to_deg(f32::acos(cos_h)))/15.0;
        let lmt = h + ra_adjusted - (0.06571 * t) - 6.622;
        let utc = lmt - lng_hour;
        let utc_adjusted = if utc <0.0 { utc + 24.0 } else if utc > 24.0 { utc - 24.0 } else { utc };
        Some((utc_adjusted * 60.0) as u16)
    }
}

fn day_of_the_year(day: u32, month: u32, year: u32) -> u16 {
    let n1 = 275 * month / 9;
    let n2 = (month + 9) / 12;
    let n3 = 1 + ((year - 4 * (year / 4) + 2) / 3);
    (n1 - (n2 * n3) + day - 30) as u16
}

fn deg_to_rad(deg: f32) -> f32 {
    (PI/180.0)*deg
}

fn rad_to_deg(rad: f32) -> f32 {
    (180.0/PI)*rad
}

#[cfg(test)]
mod tests {
    use crate::sun::{day_of_the_year, sunrise};

    #[test]
    fn first_day() {
        assert_eq!(day_of_the_year(01, 01, 2021), 1);
    }

    #[test]
    fn last_day() {
        assert_eq!(day_of_the_year(31, 12, 2021), 365);
    }

    #[test]
    fn random_day() {
        assert_eq!(day_of_the_year(03, 11, 2021), 307);
    }

    #[test]
    fn leap_day() {
        assert_eq!(day_of_the_year(29, 02, 2020), 60);
    }

    #[test]
    fn day_after_leap_day() {
        assert_eq!(day_of_the_year(01, 03, 2020), 61);
    }

    #[test]
    fn random_day_sunrise() {
        let sr = sunrise(03, 11, 2021, 24.14015, 60.05842);
        assert!(sr.is_some());
        assert_eq!(sr.unwrap(), 304);
    }

    #[test]
    fn first_day_sunrise() {
        let sr = sunrise(01, 01, 2021, 24.14015, 60.05842);
        assert!(sr.is_some());
        assert_eq!(sr.unwrap(), 389);
    }

    #[test]
    fn spring_equinox_sunrise() {
        let sr = sunrise(20, 03, 2021, 24.14015, 60.05842);
        assert!(sr.is_some());
        assert_eq!(sr.unwrap(), 224);
    }

    #[test]
    fn summer_solstice_sunrise() {
        let sr = sunrise(21, 06, 2021, 24.14015, 60.05842);
        assert!(sr.is_some());
        assert_eq!(sr.unwrap(), 1389);
    }

    #[test]
    fn autumn_equinox_sunrise() {
        let sr = sunrise(23, 09, 2021, 24.14015, 60.05842);
        assert!(sr.is_some());
        assert_eq!(sr.unwrap(), 207);
    }

    #[test]
    fn winter_solstice_sunrise() {
        let sr = sunrise(21, 12, 2021, 24.14015, 60.05842);
        assert!(sr.is_some());
        assert_eq!(sr.unwrap(), 387);
    }
}