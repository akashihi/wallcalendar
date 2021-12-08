use board::hal::datetime::Date;

// Finish holidays 2022.
//TODO Make other countries holidays
//TODO Calculate unstable holidays dates
pub fn is_holiday(date: Date) -> bool {
    (date.month == 1 && date.date == 1)
        || (date.month == 1 && date.date == 6)
        || (date.month == 4 && date.date == 15)
        || (date.month == 4 && date.date == 18)
        || (date.month == 5 && date.date == 1)
        || (date.month == 5 && date.date == 26)
        || (date.month == 6 && date.date == 24)
        || (date.month == 12 && date.date == 6)
        || (date.month == 12 && date.date == 24)
        || (date.month == 12 && date.date == 25)
        || (date.month == 12 && date.date == 26)
}
