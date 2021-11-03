#![deny(missing_docs)]
#![deny(unsafe_code)]

//! Weekday calculation using Zeller approach

/// Calculates weekday number for a date.
/// 1 - Monday, 7 - Sunday
pub fn weekday(day: u32, month: u32, year: u32) -> u8 {
    let year_adjusted = year + ((month + 9) / 12) - 1;
    let month_adjusted = (month + 9) % 12;
    let leap_year_correction = year_adjusted * 365 + (year_adjusted / 4) - (year_adjusted / 100) + (year_adjusted / 400);
    let zeller = leap_year_correction + month_adjusted * 30 + ((6 * month_adjusted + 5) / 10) + day + 1;
    ((zeller % 7) + 1) as u8
}

#[cfg(test)]
mod tests {
    use crate::weekday::weekday;

    #[test]
    fn random_wednesday() {
        assert_eq!(weekday(03, 11, 2021), 3);
    }

    #[test]
    fn monday_is_one() {
        assert_eq!(weekday(04, 10, 2021), 1);
    }

    #[test]
    fn sunday_is_seven() {
        assert_eq!(weekday(05, 12, 2021), 7);
    }

    #[test]
    fn last_day() {
        assert_eq!(weekday(31, 12, 2021), 5);
    }

    #[test]
    fn first_day() {
        assert_eq!(weekday(01, 01, 2021), 5);
    }

    #[test]
    fn leap_day() {
        assert_eq!(weekday(29, 02, 2020), 6);
    }
}