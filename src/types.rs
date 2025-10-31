use crate::consts::*;
use crate::ParseError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU16;
use std::num::NonZeroU8;

/// A year value guaranteed to be in the range 1..=MAX_YEAR (1..=9999)
/// Uses NonZeroU16 internally, so 0 is not a valid year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Year(NonZeroU16);

impl Year {
    /// Creates a new Year, validating that it's non-zero and <= MAX_YEAR
    pub fn new(value: u16) -> Result<Self, ParseError> {
        let non_zero = NonZeroU16::new(value).ok_or(ParseError::InvalidYear(value))?;
        if value > MAX_YEAR {
            return Err(ParseError::InvalidYear(value));
        }
        Ok(Self(non_zero))
    }

    /// Returns the year value as u16
    #[inline]
    pub fn get(self) -> u16 {
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

/// A month value guaranteed to be in the range 1..=MAX_MONTH (1..=12)
/// Uses NonZeroU8 internally, so 0 is not a valid month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Month(NonZeroU8);

impl Month {
    /// Creates a new Month, validating that it's non-zero and <= MAX_MONTH
    pub fn new(value: u8) -> Result<Self, ParseError> {
        let non_zero = NonZeroU8::new(value).ok_or(ParseError::InvalidMonth(value))?;
        if value > MAX_MONTH {
            return Err(ParseError::InvalidMonth(value));
        }
        Ok(Self(non_zero))
    }

    /// Returns the month value as u8
    #[inline]
    pub fn get(self) -> u8 {
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
/// Uses NonZeroU8 internally, so 0 is not a valid day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Day(NonZeroU8);

impl Day {
    /// Creates a new Day, validating that it's non-zero and valid for the given year and month
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
    pub fn get(self) -> u8 {
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
        // SAFETY: We just validated that value is >= MIN_DAY which is 1, so it's non-zero
        Ok(Self(unsafe { NonZeroU8::new_unchecked(value) }))
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

pub(crate) fn is_leap_year(year: u16) -> bool {
    (year % LEAP_YEAR_CYCLE == 0 && year % CENTURY_CYCLE != 0) || (year % GREGORIAN_CYCLE == 0)
}

pub(crate) fn days_in_month(year: u16, month: u8) -> u8 {
    debug_assert!(month != 0 && month <= MAX_MONTH);

    if month == FEBRUARY && is_leap_year(year) {
        FEBRUARY_DAYS_LEAP
    } else {
        DAYS_IN_MONTH[month as usize]
    }
}
