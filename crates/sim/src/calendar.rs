//! Gregorian calendar helpers, independent of UI and persistence.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl CalendarDate {
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if (1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day) {
            Some(Self { year, month, day })
        } else {
            None
        }
    }
    pub fn add_days(mut self, days: u32) -> Self {
        for _ in 0..days {
            self = self.next_day();
        }
        self
    }
    pub fn next_day(self) -> Self {
        if self.day < days_in_month(self.year, self.month) {
            Self { day: self.day + 1, ..self }
        } else if self.month == 12 {
            Self { year: self.year + 1, month: 1, day: 1 }
        } else {
            Self { year: self.year, month: self.month + 1, day: 1 }
        }
    }
}
pub const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn month_and_leap_rollover() {
        assert_eq!(
            CalendarDate::new(1848, 2, 28).unwrap().next_day(),
            CalendarDate::new(1848, 2, 29).unwrap()
        );
        assert_eq!(
            CalendarDate::new(1847, 2, 28).unwrap().next_day(),
            CalendarDate::new(1847, 3, 1).unwrap()
        );
        assert_eq!(
            CalendarDate::new(1848, 12, 31).unwrap().next_day(),
            CalendarDate::new(1849, 1, 1).unwrap()
        );
    }
}
