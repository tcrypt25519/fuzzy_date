use crate::consts::{
    CENTURY_CYCLE, DAYS_IN_MONTH, FEBRUARY, FEBRUARY_DAYS_LEAP, GREGORIAN_CYCLE, LEAP_YEAR_CYCLE,
    MAX_MONTH, MAX_YEAR, MIN_DAY,
};
use crate::ParseError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU16;
use std::num::NonZeroU8;

/// A year value guaranteed to be in the range `1..=MAX_YEAR` (1..=9999)
/// Uses `NonZeroU16` internally, so 0 is not a valid year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Year(NonZeroU16);

impl Year {
    /// Creates a new Year, validating that it's non-zero and <= `MAX_YEAR`
    ///
    /// # Errors
    /// Returns `ParseError::InvalidYear` if the value is 0 or > `MAX_YEAR`.
    pub fn new(value: u16) -> Result<Self, ParseError> {
        let non_zero = NonZeroU16::new(value).ok_or(ParseError::InvalidYear(value))?;
        if value > MAX_YEAR {
            return Err(ParseError::InvalidYear(value));
        }
        Ok(Self(non_zero))
    }

    /// Returns the year value as u16
    #[inline]
    pub const fn get(self) -> u16 {
        self.0.get()
    }
}

impl TryFrom<u16> for Year {
    type Error = ParseError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Year> for u16 {
    fn from(year: Year) -> Self {
        year.0.get()
    }
}

impl fmt::Display for Year {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A month value guaranteed to be in the range `1..=MAX_MONTH` (1..=12)
/// Uses `NonZeroU8` internally, so 0 is not a valid month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Month(NonZeroU8);

impl Month {
    /// Creates a new Month, validating that it's non-zero and <= `MAX_MONTH`
    ///
    /// # Errors
    /// Returns `ParseError::InvalidMonth` if the value is 0 or > `MAX_MONTH`.
    pub fn new(value: u8) -> Result<Self, ParseError> {
        let non_zero = NonZeroU8::new(value).ok_or(ParseError::InvalidMonth(value))?;
        if value > MAX_MONTH {
            return Err(ParseError::InvalidMonth(value));
        }
        Ok(Self(non_zero))
    }

    /// Returns the month value as u8
    #[inline]
    pub const fn get(self) -> u8 {
        self.0.get()
    }
}

impl TryFrom<u8> for Month {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Month> for u8 {
    fn from(month: Month) -> Self {
        month.0.get()
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A day value guaranteed to be valid for a given year and month
/// Uses `NonZeroU8` internally, so 0 is not a valid day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Day(NonZeroU8);

impl Day {
    /// Creates a new Day, validating that it's non-zero and valid for the given year and month
    ///
    /// # Errors
    /// Returns `ParseError::InvalidDay` if the value is 0 or invalid for the given year and month.
    pub fn new(value: u8, year: u16, month: u8) -> Result<Self, ParseError> {
        let non_zero = NonZeroU8::new(value).ok_or(ParseError::InvalidDay {
            month,
            day: value,
            year,
        })?;

        let max_day = days_in_month(year, month);
        if value > max_day {
            return Err(ParseError::InvalidDay {
                month,
                day: value,
                year,
            });
        }

        Ok(Self(non_zero))
    }

    /// Returns the day value as u8
    #[inline]
    pub const fn get(self) -> u8 {
        self.0.get()
    }
}

impl TryFrom<u8> for Day {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        // Can't validate without year/month context, so just check minimum
        if value < MIN_DAY {
            return Err(ParseError::InvalidDay {
                month: 0,
                day: value,
                year: 0,
            });
        }
        // Since we validated value >= MIN_DAY (which is 1), value is non-zero
        let non_zero = NonZeroU8::new(value).ok_or(ParseError::InvalidDay {
            month: 0,
            day: value,
            year: 0,
        })?;
        Ok(Self(non_zero))
    }
}

impl From<Day> for u8 {
    fn from(day: Day) -> Self {
        day.0.get()
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Helper functions

pub const fn is_leap_year(year: u16) -> bool {
    (year % LEAP_YEAR_CYCLE == 0 && year % CENTURY_CYCLE != 0) || (year % GREGORIAN_CYCLE == 0)
}

pub const fn days_in_month(year: u16, month: u8) -> u8 {
    debug_assert!(month != 0 && month <= MAX_MONTH);

    if month == FEBRUARY && is_leap_year(year) {
        FEBRUARY_DAYS_LEAP
    } else {
        DAYS_IN_MONTH[month as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_new_valid() {
        assert!(Year::new(1).is_ok());
        assert!(Year::new(2000).is_ok());
        assert!(Year::new(9999).is_ok());
    }

    #[test]
    fn test_year_new_invalid_zero() {
        let result = Year::new(0);
        assert!(matches!(result, Err(ParseError::InvalidYear(0))));
    }

    #[test]
    fn test_year_new_invalid_too_large() {
        let result = Year::new(10000);
        assert!(matches!(result, Err(ParseError::InvalidYear(10000))));
    }

    #[test]
    fn test_year_get() {
        let year = Year::new(2024).unwrap();
        assert_eq!(year.get(), 2024);
    }

    #[test]
    fn test_year_display() {
        let year = Year::new(2024).unwrap();
        assert_eq!(year.to_string(), "2024");
    }

    #[test]
    fn test_year_try_from_u16() {
        let year: Year = 2024.try_into().unwrap();
        assert_eq!(year.get(), 2024);

        let result: Result<Year, _> = 0.try_into();
        assert!(result.is_err());

        let result: Result<Year, _> = 10000.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_year_into_u16() {
        let year = Year::new(2024).unwrap();
        let value: u16 = year.into();
        assert_eq!(value, 2024);
    }

    #[test]
    fn test_year_ordering() {
        let y1 = Year::new(2020).unwrap();
        let y2 = Year::new(2024).unwrap();
        assert!(y1 < y2);
        assert!(y2 > y1);
        assert_eq!(y1, y1);
    }

    #[test]
    fn test_year_serde() {
        let year = Year::new(2024).unwrap();
        let json = serde_json::to_string(&year).unwrap();
        assert_eq!(json, "2024");

        let parsed: Year = serde_json::from_str(&json).unwrap();
        assert_eq!(year, parsed);
    }

    #[test]
    fn test_month_new_valid() {
        for m in 1..=12 {
            assert!(Month::new(m).is_ok(), "Month {m} should be valid");
        }
    }

    #[test]
    fn test_month_new_invalid_zero() {
        let result = Month::new(0);
        assert!(matches!(result, Err(ParseError::InvalidMonth(0))));
    }

    #[test]
    fn test_month_new_invalid_too_large() {
        let result = Month::new(13);
        assert!(matches!(result, Err(ParseError::InvalidMonth(13))));

        let result = Month::new(255);
        assert!(matches!(result, Err(ParseError::InvalidMonth(255))));
    }

    #[test]
    fn test_month_get() {
        let month = Month::new(8).unwrap();
        assert_eq!(month.get(), 8);
    }

    #[test]
    fn test_month_display() {
        let month = Month::new(8).unwrap();
        assert_eq!(month.to_string(), "8");
    }

    #[test]
    fn test_month_try_from_u8() {
        let month: Month = 8.try_into().unwrap();
        assert_eq!(month.get(), 8);

        let result: Result<Month, _> = 0.try_into();
        assert!(result.is_err());

        let result: Result<Month, _> = 13.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_month_into_u8() {
        let month = Month::new(8).unwrap();
        let value: u8 = month.into();
        assert_eq!(value, 8);
    }

    #[test]
    fn test_month_ordering() {
        let m1 = Month::new(3).unwrap();
        let m2 = Month::new(8).unwrap();
        assert!(m1 < m2);
        assert!(m2 > m1);
        assert_eq!(m1, m1);
    }

    #[test]
    fn test_month_serde() {
        let month = Month::new(8).unwrap();
        let json = serde_json::to_string(&month).unwrap();
        assert_eq!(json, "8");

        let parsed: Month = serde_json::from_str(&json).unwrap();
        assert_eq!(month, parsed);
    }

    #[test]
    fn test_day_new_valid() {
        // January - 31 days
        assert!(Day::new(1, 2024, 1).is_ok());
        assert!(Day::new(31, 2024, 1).is_ok());

        // February non-leap - 28 days
        assert!(Day::new(28, 2023, 2).is_ok());
        assert!(Day::new(29, 2023, 2).is_err());

        // February leap year - 29 days
        assert!(Day::new(29, 2024, 2).is_ok());
        assert!(Day::new(30, 2024, 2).is_err());

        // April - 30 days
        assert!(Day::new(30, 2024, 4).is_ok());
        assert!(Day::new(31, 2024, 4).is_err());
    }

    #[test]
    fn test_day_new_invalid_zero() {
        let result = Day::new(0, 2024, 1);
        assert!(matches!(result, Err(ParseError::InvalidDay { .. })));
    }

    #[test]
    fn test_day_new_invalid_too_large() {
        // 32 is invalid for January
        let result = Day::new(32, 2024, 1);
        assert!(matches!(
            result,
            Err(ParseError::InvalidDay {
                month: 1,
                day: 32,
                year: 2024
            })
        ));
    }

    #[test]
    fn test_day_get() {
        let day = Day::new(15, 2024, 8).unwrap();
        assert_eq!(day.get(), 15);
    }

    #[test]
    fn test_day_display() {
        let day = Day::new(15, 2024, 8).unwrap();
        assert_eq!(day.to_string(), "15");
    }

    #[test]
    fn test_day_try_from_u8() {
        // Valid day (context-free validation)
        let day: Day = 15.try_into().unwrap();
        assert_eq!(day.get(), 15);

        // Zero is invalid
        let result: Result<Day, _> = 0.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_day_into_u8() {
        let day = Day::new(15, 2024, 8).unwrap();
        let value: u8 = day.into();
        assert_eq!(value, 15);
    }

    #[test]
    fn test_day_ordering() {
        let d1 = Day::new(10, 2024, 8).unwrap();
        let d2 = Day::new(20, 2024, 8).unwrap();
        assert!(d1 < d2);
        assert!(d2 > d1);
        assert_eq!(d1, d1);
    }

    #[test]
    fn test_day_serde() {
        let day = Day::new(15, 2024, 8).unwrap();
        let json = serde_json::to_string(&day).unwrap();
        assert_eq!(json, "15");

        let parsed: Day = serde_json::from_str(&json).unwrap();
        assert_eq!(day, parsed);
    }

    #[test]
    fn test_is_leap_year_cases() {
        struct TestCase {
            year: u16,
            is_leap: bool,
            description: &'static str,
        }

        let cases = [
            // Divisible by 4
            TestCase {
                year: 2020,
                is_leap: true,
                description: "divisible by 4",
            },
            TestCase {
                year: 2024,
                is_leap: true,
                description: "divisible by 4",
            },
            TestCase {
                year: 2021,
                is_leap: false,
                description: "not divisible by 4",
            },
            TestCase {
                year: 2023,
                is_leap: false,
                description: "not divisible by 4",
            },
            // Century years not divisible by 400
            TestCase {
                year: 1900,
                is_leap: false,
                description: "century not divisible by 400",
            },
            TestCase {
                year: 2100,
                is_leap: false,
                description: "century not divisible by 400",
            },
            TestCase {
                year: 2200,
                is_leap: false,
                description: "century not divisible by 400",
            },
            TestCase {
                year: 2300,
                is_leap: false,
                description: "century not divisible by 400",
            },
            // Divisible by 400
            TestCase {
                year: 2000,
                is_leap: true,
                description: "divisible by 400",
            },
            TestCase {
                year: 2400,
                is_leap: true,
                description: "divisible by 400",
            },
        ];

        for case in &cases {
            assert_eq!(
                is_leap_year(case.year),
                case.is_leap,
                "Year {} ({}): expected {}",
                case.year,
                case.description,
                if case.is_leap {
                    "leap year"
                } else {
                    "not leap year"
                }
            );
        }
    }

    #[test]
    fn test_days_in_month_31_day_months() {
        for month in [1, 3, 5, 7, 8, 10, 12] {
            assert_eq!(
                days_in_month(2024, month),
                31,
                "Month {month} should have 31 days"
            );
        }
    }

    #[test]
    fn test_days_in_month_30_day_months() {
        for month in [4, 6, 9, 11] {
            assert_eq!(
                days_in_month(2024, month),
                30,
                "Month {month} should have 30 days"
            );
        }
    }

    #[test]
    fn test_days_in_month_february_non_leap() {
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2021, 2), 28);
        assert_eq!(
            days_in_month(1900, 2),
            28,
            "Century year not divisible by 400"
        );
    }

    #[test]
    fn test_days_in_month_february_leap() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2020, 2), 29);
        assert_eq!(days_in_month(2000, 2), 29, "Century year divisible by 400");
    }

    #[test]
    fn test_all_months_have_valid_days() {
        // Verify all months in DAYS_IN_MONTH array are correct for a non-leap year
        let expected = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for month in 1..=12 {
            assert_eq!(
                days_in_month(2023, month),
                expected[month as usize],
                "Month {month} has incorrect day count"
            );
        }
    }
}
