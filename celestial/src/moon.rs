#![deny(missing_docs)]
#![deny(unsafe_code)]

//! Moon phase calculation using Julian Day approach

/// Calculates moon phase for a date.
/// 1 - New moon, 5 - Full moon, 8 - Waning Crescent
pub fn moon_phase(day: u32, month: u32, year: u32) -> u8 {
    let a = (year/100) as i32;
    let b = (a/4) as i32;
    let c = 2 - a + b;
    let e = (365.25 * (year as f32 + 4716.0)) as i32;
    let f = (30.6001 * (month as f32 + 1.0)) as i32;
    let jd = (c + day as i32 + e + f) as f32 - 1524.5;
    let new_moons = jd/29.53;
    let cycle_length = new_moons - (new_moons as i32) as f32;
    if cycle_length >= 0.0 && cycle_length < 0.125 { 1 }
    else if cycle_length>=0.125 && cycle_length < 0.25 { 2 }
    else if cycle_length>=0.25 && cycle_length < 0.375 { 3 }
    else if cycle_length>=0.375 && cycle_length < 0.5 { 4 }
    else if cycle_length>=0.5 && cycle_length < 0.625 { 5 }
    else if cycle_length>=0.625 && cycle_length < 0.75 { 6 }
    else if cycle_length>=0.75 && cycle_length < 0.875 { 7 }
    else /*if cycle_length>=0.875*/ { 8 }
}

#[cfg(test)]
mod tests {
    use crate::moon::moon_phase;

    #[test]
    fn new_moon() {
        assert_eq!(moon_phase(06, 11, 2021), 1);
    }

    #[test]
    fn first_quarter() {
        assert_eq!(moon_phase(14, 11, 2021), 3);
    }

    #[test]
    fn full_moon() {
        assert_eq!(moon_phase(21, 11, 2021), 5);
    }

    #[test]
    fn third_quarter() {
        assert_eq!(moon_phase(29, 11, 2021), 7);
    }

}