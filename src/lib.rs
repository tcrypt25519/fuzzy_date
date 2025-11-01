mod consts;
mod prelude;
mod range;
mod types;

pub use consts::*;
pub use range::{FuzzyDateRange, RangeError};
pub use types::{Day, Month, Year};

use crate::prelude::*;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::str::FromStr;
use types::days_in_month;

/// Represents a date with varying levels of precision.
/// This allows representing dates where only some components are known,
/// without fabricating missing data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum FuzzyDate {
    /// Full date with day, month, and year
    #[display(fmt = "{:04}-{:02}-{:02}", "year.get()", "month.get()", "day.get()")]
    Day {
        year: types::Year,
        month: types::Month,
        day: types::Day,
    },
    /// Month and year only
    #[display(fmt = "{:04}-{:02}", "year.get()", "month.get()")]
    Month {
        year: types::Year,
        month: types::Month,
    },
    /// Year only
    #[display(fmt = "{:04}", "year.get()")]
    Year { year: types::Year },
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum ParseError {
    #[display(fmt = "Invalid date format: {_0}")]
    InvalidFormat(String),
    #[display(fmt = "Invalid year: {} (must be 1-{})", "_0", MAX_YEAR)]
    InvalidYear(u16),
    #[display(fmt = "Invalid month: {} (must be 1-{})", "_0", MAX_MONTH)]
    InvalidMonth(u8),
    #[display(fmt = "Invalid day {day} for month {year}-{month:02}")]
    InvalidDay { month: u8, day: u8, year: u16 },
    #[display(fmt = "Empty date string")]
    EmptyInput,
}

impl std::error::Error for ParseError {}

impl FuzzyDate {
    /// Creates a new full date (types are already validated)
    pub fn new_day(
        year: types::Year,
        month: types::Month,
        day: types::Day,
    ) -> Result<Self, ParseError> {
        Ok(Self::Day { year, month, day })
    }

    /// Creates a new month-year date (types are already validated)
    pub fn new_month(year: types::Year, month: types::Month) -> Result<Self, ParseError> {
        Ok(Self::Month { year, month })
    }

    /// Creates a new year-only date (type is already validated)
    pub fn new_year(year: types::Year) -> Result<Self, ParseError> {
        Ok(Self::Year { year })
    }

    /// Returns the day component if present (as u8 for convenience)
    pub fn day(&self) -> Option<u8> {
        match self {
            Self::Day { day, .. } => Some(day.get()),
            Self::Month { .. } | Self::Year { .. } => None,
        }
    }

    /// Returns the month component if present (as u8 for convenience)
    pub fn month(&self) -> Option<u8> {
        match self {
            Self::Day { month, .. } | Self::Month { month, .. } => Some(month.get()),
            Self::Year { .. } => None,
        }
    }

    /// Returns the year component (always present)
    pub fn year(&self) -> u16 {
        match self {
            Self::Day { year, .. } | Self::Month { year, .. } | Self::Year { year } => year.get(),
        }
    }

    /// Returns the Day type if present
    pub fn day_typed(&self) -> Option<types::Day> {
        match self {
            Self::Day { day, .. } => Some(*day),
            Self::Month { .. } | Self::Year { .. } => None,
        }
    }

    /// Returns the Month type if present
    pub fn month_typed(&self) -> Option<types::Month> {
        match self {
            Self::Day { month, .. } | Self::Month { month, .. } => Some(*month),
            Self::Year { .. } => None,
        }
    }

    /// Returns the Year type (always present)
    pub fn year_typed(&self) -> types::Year {
        match self {
            Self::Day { year, .. } | Self::Month { year, .. } | Self::Year { year } => *year,
        }
    }

    /// Converts to database columns: (year, month, day)
    pub fn to_columns(&self) -> (u16, Option<u8>, Option<u8>) {
        match *self {
            Self::Day { year, month, day } => (year.get(), Some(month.get()), Some(day.get())),
            Self::Month { year, month } => (year.get(), Some(month.get()), None),
            Self::Year { year } => (year.get(), None, None),
        }
    }

    /// Creates from database columns: (year, month, day)
    pub fn from_columns(year: u16, month: Option<u8>, day: Option<u8>) -> Result<Self, ParseError> {
        match (month, day) {
            (Some(m), Some(d)) => {
                let year_nz = Self::validate_and_convert_year(year)?;
                let month_nz = Self::validate_and_convert_month(m)?;
                let day_nz = Self::validate_and_convert_day(year, m, d)?;
                Ok(Self::Day {
                    year: year_nz,
                    month: month_nz,
                    day: day_nz,
                })
            }
            (Some(m), None) => {
                let year_nz = Self::validate_and_convert_year(year)?;
                let month_nz = Self::validate_and_convert_month(m)?;
                Ok(Self::Month {
                    year: year_nz,
                    month: month_nz,
                })
            }
            (None, None) => {
                let year_nz = Self::validate_and_convert_year(year)?;
                Ok(Self::Year { year: year_nz })
            }
            (None, Some(d)) => Err(ParseError::InvalidFormat(format!(
                "Cannot have day {} without month",
                d
            ))),
        }
    }
}

// --- helpers for bounds / validation ---
fn next_month(year: u16, month: u8) -> Option<(u16, u8)> {
    debug_assert!(month != 0 && month <= MAX_MONTH);
    if month == DECEMBER {
        // Check both overflow and our MAX_YEAR limit
        if year >= MAX_YEAR {
            None
        } else {
            Some((year + 1, JANUARY))
        }
    } else {
        Some((year, month + 1))
    }
}

fn next_day(year: u16, month: u8, day: u8) -> Option<(u16, u8, u8)> {
    let max = days_in_month(year, month);
    if day < max {
        Some((year, month, day + 1))
    } else {
        // roll to first of next month (respects MAX_YEAR limit)
        next_month(year, month).map(|(ny, nm)| (ny, nm, MIN_DAY))
    }
}

impl FromStr for FuzzyDate {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        // Strictly enforce delimiters: DATE_SEPARATOR for ISO, MONTH_FIRST_SEPARATOR for month-first
        let has_hyphen = trimmed.contains(DATE_SEPARATOR);
        let has_slash = trimmed.contains(MONTH_FIRST_SEPARATOR);

        if has_hyphen && has_slash {
            return Err(ParseError::InvalidFormat(format!(
                "Mixed delimiters ({} and {})",
                DATE_SEPARATOR, MONTH_FIRST_SEPARATOR
            )));
        }

        if has_hyphen {
            // ISO format: YYYY or YYYY-MM or YYYY-MM-DD
            let parts: Vec<&str> = trimmed.split(DATE_SEPARATOR).map(|p| p.trim()).collect();
            match parts.len() {
                1 => Self::parse_year_only(parts[0]),
                2 => Self::parse_iso_month_year(&parts),
                3 => Self::parse_iso_full_date(&parts),
                _ => Err(ParseError::InvalidFormat(format!(
                    "Too many {} separators: expected 0-2, found {}",
                    DATE_SEPARATOR,
                    parts.len() - 1
                ))),
            }
        } else if has_slash {
            // Month-first format: MM/YYYY or MM/DD/YYYY
            let parts: Vec<&str> = trimmed
                .split(MONTH_FIRST_SEPARATOR)
                .map(|p| p.trim())
                .collect();
            match parts.len() {
                2 => Self::parse_month_year(&parts),
                3 => Self::parse_full_date(&parts),
                _ => Err(ParseError::InvalidFormat(format!(
                    "Too many {} separators: expected 1-2, found {}",
                    MONTH_FIRST_SEPARATOR,
                    parts.len() - 1
                ))),
            }
        } else {
            // No delimiter, bare year
            Self::parse_year_only(trimmed)
        }
    }
}

impl FuzzyDate {
    /// Helper to parse u16 with better error messages
    fn parse_u16(s: &str) -> Result<u16, ParseError> {
        s.parse::<u16>()
            .map_err(|_| ParseError::InvalidFormat(s.to_owned()))
    }

    /// Helper to parse u8 with better error messages
    fn parse_u8(s: &str) -> Result<u8, ParseError> {
        s.parse::<u8>()
            .map_err(|_| ParseError::InvalidFormat(s.to_owned()))
    }

    /// Validates and creates a Year type
    fn validate_and_convert_year(year: u16) -> Result<types::Year, ParseError> {
        types::Year::new(year)
    }

    /// Validates and creates a Month type
    fn validate_and_convert_month(month: u8) -> Result<types::Month, ParseError> {
        types::Month::new(month)
    }

    /// Validates and creates a Day type
    fn validate_and_convert_day(year: u16, month: u8, day: u8) -> Result<types::Day, ParseError> {
        types::Day::new(day, year, month)
    }

    fn parse_iso_month_year(parts: &[&str]) -> Result<Self, ParseError> {
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat(parts.join("-")));
        }
        // Parse components - InvalidFormat if not numeric
        let year_u16 = Self::parse_u16(parts[0])?;
        let month_u8 = Self::parse_u8(parts[1])?;

        // Validate and convert to NonZero types
        let year = Self::validate_and_convert_year(year_u16)?;
        let month = Self::validate_and_convert_month(month_u8)?;

        Ok(Self::Month { year, month })
    }

    fn parse_iso_full_date(parts: &[&str]) -> Result<Self, ParseError> {
        if parts.len() != 3 {
            return Err(ParseError::InvalidFormat(parts.join("-")));
        }
        // Parse components - InvalidFormat if not numeric
        let year_u16 = Self::parse_u16(parts[0])?;
        let month_u8 = Self::parse_u8(parts[1])?;
        let day_u8 = Self::parse_u8(parts[2])?;

        // Validate and convert to NonZero types
        let year = Self::validate_and_convert_year(year_u16)?;
        let month = Self::validate_and_convert_month(month_u8)?;
        let day = Self::validate_and_convert_day(year_u16, month_u8, day_u8)?;

        Ok(Self::Day { year, month, day })
    }

    fn parse_year_only(s: &str) -> Result<Self, ParseError> {
        // Parse as number - InvalidFormat if not numeric
        let year_u16 = Self::parse_u16(s)?;

        // Validate and convert to NonZero type
        let year = Self::validate_and_convert_year(year_u16)?;

        Ok(Self::Year { year })
    }

    fn parse_month_year(parts: &[&str]) -> Result<Self, ParseError> {
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat(parts.join("/")));
        }
        // Parse components - InvalidFormat if not numeric
        let month_u8 = Self::parse_u8(parts[0])?;
        let year_u16 = Self::parse_u16(parts[1])?;

        // Validate and convert to NonZero types
        let month = Self::validate_and_convert_month(month_u8)?;
        let year = Self::validate_and_convert_year(year_u16)?;

        Ok(Self::Month { year, month })
    }

    fn parse_full_date(parts: &[&str]) -> Result<Self, ParseError> {
        if parts.len() != 3 {
            return Err(ParseError::InvalidFormat(parts.join("/")));
        }
        // Parse components - InvalidFormat if not numeric
        let month_u8 = Self::parse_u8(parts[0])?;
        let day_u8 = Self::parse_u8(parts[1])?;
        let year_u16 = Self::parse_u16(parts[2])?;

        // Validate and convert to NonZero types
        let year = Self::validate_and_convert_year(year_u16)?;
        let month = Self::validate_and_convert_month(month_u8)?;
        let day = Self::validate_and_convert_day(year_u16, month_u8, day_u8)?;

        Ok(Self::Day { year, month, day })
    }
}

impl FuzzyDate {
    /// Earliest concrete (year, month, day) represented by this value.
    pub fn lower_bound(&self) -> (u16, u8, u8) {
        match *self {
            FuzzyDate::Day { year, month, day } => (year.get(), month.get(), day.get()),
            FuzzyDate::Month { year, month } => (year.get(), month.get(), MIN_DAY),
            FuzzyDate::Year { year } => (year.get(), JANUARY, MIN_DAY),
        }
    }

    /// Latest concrete (year, month, day) represented by this value (inclusive).
    pub fn upper_bound_inclusive(&self) -> (u16, u8, u8) {
        match *self {
            FuzzyDate::Day { year, month, day } => (year.get(), month.get(), day.get()),
            FuzzyDate::Month { year, month } => (
                year.get(),
                month.get(),
                days_in_month(year.get(), month.get()),
            ),
            FuzzyDate::Year { year } => (year.get(), DECEMBER, DAYS_IN_MONTH[DECEMBER as usize]),
        }
    }

    /// Exclusive upper bound (year, month, day).
    /// Returns `None` if it would overflow `MAX_YEAR` limit.
    pub fn upper_bound_exclusive(&self) -> Option<(u16, u8, u8)> {
        match *self {
            FuzzyDate::Day { year, month, day } => next_day(year.get(), month.get(), day.get()),
            FuzzyDate::Month { year, month } => {
                next_month(year.get(), month.get()).map(|(ny, nm)| (ny, nm, MIN_DAY))
            }
            FuzzyDate::Year { year } => {
                let y = year.get();
                if y >= MAX_YEAR {
                    None
                } else {
                    Some((y + 1, JANUARY, MIN_DAY))
                }
            }
        }
    }

    /// Rank used for ordering ties on the same `lower_bound`:
    /// less precise comes first: Year < Month < Day.
    #[inline]
    fn precision_rank(&self) -> u8 {
        match *self {
            FuzzyDate::Year { .. } => 0,
            FuzzyDate::Month { .. } => 1,
            FuzzyDate::Day { .. } => 2,
        }
    }
}

impl PartialOrd for FuzzyDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FuzzyDate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare earliest possible concrete date first…
        let a = self.lower_bound();
        let b = other.lower_bound();
        match a.cmp(&b) {
            Ordering::Equal => {
                // …then break ties by precision (less precise first).
                self.precision_rank().cmp(&other.precision_rank())
            }
            ord => ord,
        }
    }
}

impl TryFrom<(u16, Option<u8>, Option<u8>)> for FuzzyDate {
    type Error = ParseError;

    fn try_from(value: (u16, Option<u8>, Option<u8>)) -> Result<Self, Self::Error> {
        Self::from_columns(value.0, value.1, value.2)
    }
}

impl serde::Serialize for FuzzyDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for FuzzyDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_date() {
        let date = "08/15/1991".parse::<FuzzyDate>().unwrap();
        assert_eq!(
            date,
            FuzzyDate::Day {
                year: Year::new(1991).unwrap(),
                month: Month::new(8).unwrap(),
                day: Day::new(15, 1991, 8).unwrap()
            }
        );
        assert_eq!(date.year(), 1991);
        assert_eq!(date.month(), Some(8));
        assert_eq!(date.day(), Some(15));
    }

    #[test]
    fn test_parse_iso_full_date() {
        let date = "1991-08-15".parse::<FuzzyDate>().unwrap();
        assert_eq!(
            date,
            FuzzyDate::Day {
                year: Year::new(1991).unwrap(),
                month: Month::new(8).unwrap(),
                day: Day::new(15, 1991, 8).unwrap()
            }
        );
    }

    #[test]
    fn test_parse_month_year() {
        let date = "08/1991".parse::<FuzzyDate>().unwrap();
        assert_eq!(
            date,
            FuzzyDate::Month {
                year: Year::new(1991).unwrap(),
                month: Month::new(8).unwrap()
            }
        );
        assert_eq!(date.year(), 1991);
        assert_eq!(date.month(), Some(8));
        assert_eq!(date.day(), None);
    }

    #[test]
    fn test_parse_iso_month_year() {
        let date = "1991-08".parse::<FuzzyDate>().unwrap();
        assert_eq!(
            date,
            FuzzyDate::Month {
                year: Year::new(1991).unwrap(),
                month: Month::new(8).unwrap()
            }
        );
    }

    #[test]
    fn test_parse_year_only() {
        let date = "1991".parse::<FuzzyDate>().unwrap();
        assert_eq!(
            date,
            FuzzyDate::Year {
                year: Year::new(1991).unwrap()
            }
        );
        assert_eq!(date.year(), 1991);
        assert_eq!(date.month(), None);
        assert_eq!(date.day(), None);
    }

    #[test]
    fn test_parse_with_hyphens() {
        // This test needs to be updated - hyphens are now strictly ISO format
        // "08-15-1991" is not valid ISO (should be YYYY-MM-DD)
        let result = "08-15-1991".parse::<FuzzyDate>();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_whitespace() {
        let date = " 08 / 1991 ".parse::<FuzzyDate>().unwrap();
        assert_eq!(
            date,
            FuzzyDate::Month {
                year: Year::new(1991).unwrap(),
                month: Month::new(8).unwrap()
            }
        );
    }

    #[test]
    fn test_invalid_month() {
        let result = "13/1991".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidMonth(13))));
    }

    #[test]
    fn test_invalid_day() {
        let result = "02/30/2020".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidDay { .. })));
    }

    #[test]
    fn test_leap_year() {
        // 2020 is a leap year
        let date = "02/29/2020".parse::<FuzzyDate>().unwrap();
        assert_eq!(
            date,
            FuzzyDate::Day {
                year: Year::new(2020).unwrap(),
                month: Month::new(2).unwrap(),
                day: Day::new(29, 2020, 2).unwrap()
            }
        );

        // 2021 is not a leap year
        let result = "02/29/2021".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidDay { .. })));
    }

    #[test]
    fn test_display() {
        let full = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
            day: Day::new(15, 1991, 8).unwrap(),
        };
        assert_eq!(full.to_string(), "1991-08-15");

        let month_year = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
        };
        assert_eq!(month_year.to_string(), "1991-08");

        let year = FuzzyDate::Year {
            year: Year::new(1991).unwrap(),
        };
        assert_eq!(year.to_string(), "1991");
    }

    #[test]
    fn test_bounds_year() {
        let d = FuzzyDate::Year {
            year: Year::new(1991).unwrap(),
        };
        assert_eq!(d.lower_bound(), (1991, 1, 1));
        assert_eq!(d.upper_bound_exclusive(), Some((1992, 1, 1)));
    }

    #[test]
    fn test_bounds_month_year() {
        let d = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
        };
        assert_eq!(d.lower_bound(), (1991, 8, 1));
        assert_eq!(d.upper_bound_exclusive(), Some((1991, 9, 1)));
    }

    #[test]
    fn test_bounds_full_rollover_and_leap() {
        let d1 = FuzzyDate::Day {
            year: Year::new(2020).unwrap(),
            month: Month::new(2).unwrap(),
            day: Day::new(29, 2020, 2).unwrap(),
        };
        assert_eq!(d1.lower_bound(), (2020, 2, 29));
        assert_eq!(d1.upper_bound_exclusive(), Some((2020, 3, 1)));
        let d2 = FuzzyDate::Day {
            year: Year::new(2021).unwrap(),
            month: Month::new(12).unwrap(),
            day: Day::new(31, 2021, 12).unwrap(),
        };
        assert_eq!(d2.upper_bound_exclusive(), Some((2022, 1, 1)));
    }

    #[test]
    fn test_ordering() {
        let d1 = FuzzyDate::Year {
            year: Year::new(1990).unwrap(),
        };
        let d2 = FuzzyDate::Year {
            year: Year::new(1991).unwrap(),
        };
        assert!(d1 < d2);

        let d3 = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
        };
        assert!(d2 < d3); // Year-only is less specific, comes before month-year

        let d4 = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
            day: Day::new(15, 1991, 8).unwrap(),
        };
        assert!(d3 < d4); // Month-year comes before full date in same month
    }

    #[test]
    fn test_ordering_same_lower_bound_tiebreaker() {
        // Same lower bound (1991-01-01) but different precision
        let year = FuzzyDate::Year {
            year: Year::new(1991).unwrap(),
        };
        let month = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(1).unwrap(),
        };
        let full = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(1).unwrap(),
            day: Day::new(1, 1991, 1).unwrap(),
        };
        assert!(year < month);
        assert!(month < full);
        // Sanity: anything before the lower bound should still come first
        let prev_day = FuzzyDate::Day {
            year: Year::new(1990).unwrap(),
            month: Month::new(12).unwrap(),
            day: Day::new(31, 1990, 12).unwrap(),
        };
        assert!(prev_day < year);
    }

    #[test]
    fn test_serde() {
        let date = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
        };
        let json = serde_json::to_string(&date).unwrap();
        let parsed: FuzzyDate = serde_json::from_str(&json).unwrap();
        assert_eq!(date, parsed);
    }

    #[test]
    fn test_year_bounds() {
        // Test year 0 is invalid
        let result = "0".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidYear(0))));

        // Test year 10000 is invalid
        let result = "10000".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidYear(10000))));

        // Test year 9999 is valid
        let result = "9999".parse::<FuzzyDate>();
        assert!(result.is_ok());

        // Test year 1 is valid
        let result = "1".parse::<FuzzyDate>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_delimiter_strictness() {
        // ISO format must use hyphens
        assert!("1991-08-15".parse::<FuzzyDate>().is_ok());
        assert!("1991-08".parse::<FuzzyDate>().is_ok());

        // Month-first format must use slashes
        assert!("08/15/1991".parse::<FuzzyDate>().is_ok());
        assert!("08/1991".parse::<FuzzyDate>().is_ok());

        // Mixed delimiters are invalid
        let result = "1991-08/15".parse::<FuzzyDate>();
        assert!(result.is_err());

        // Month-first format with hyphens is now invalid
        let result = "08-15-1991".parse::<FuzzyDate>();
        assert!(result.is_err());
    }

    #[test]
    fn test_upper_bound_at_year_limit() {
        // Year 9999 should have None as upper bound (can't go to 10000)
        let d = FuzzyDate::Year {
            year: Year::new(9999).unwrap(),
        };
        assert_eq!(d.upper_bound_exclusive(), None);

        // December 9999 should also be None
        let d = FuzzyDate::Month {
            year: Year::new(9999).unwrap(),
            month: Month::new(12).unwrap(),
        };
        assert_eq!(d.upper_bound_exclusive(), None);

        // Last day of 9999 should be None
        let d = FuzzyDate::Day {
            year: Year::new(9999).unwrap(),
            month: Month::new(12).unwrap(),
            day: Day::new(31, 9999, 12).unwrap(),
        };
        assert_eq!(d.upper_bound_exclusive(), None);
    }

    #[test]
    fn test_upper_bound_inclusive() {
        // Full date: same as itself
        let d = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
            day: Day::new(15, 1991, 8).unwrap(),
        };
        assert_eq!(d.upper_bound_inclusive(), (1991, 8, 15));

        // Month: last day of month
        let d = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
        };
        assert_eq!(d.upper_bound_inclusive(), (1991, 8, 31));

        // February leap year
        let d = FuzzyDate::Month {
            year: Year::new(2020).unwrap(),
            month: Month::new(2).unwrap(),
        };
        assert_eq!(d.upper_bound_inclusive(), (2020, 2, 29));

        // February non-leap year
        let d = FuzzyDate::Month {
            year: Year::new(2021).unwrap(),
            month: Month::new(2).unwrap(),
        };
        assert_eq!(d.upper_bound_inclusive(), (2021, 2, 28));

        // Year: December 31st
        let d = FuzzyDate::Year {
            year: Year::new(1991).unwrap(),
        };
        assert_eq!(d.upper_bound_inclusive(), (1991, 12, 31));
    }

    #[test]
    fn test_to_columns_and_from_columns() {
        // Full date
        let date = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
            day: Day::new(15, 1991, 8).unwrap(),
        };
        let (y, m, d) = date.to_columns();
        assert_eq!((y, m, d), (1991, Some(8), Some(15)));
        let restored = FuzzyDate::from_columns(y, m, d).unwrap();
        assert_eq!(date, restored);

        // Month year
        let date = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
        };
        let (y, m, d) = date.to_columns();
        assert_eq!((y, m, d), (1991, Some(8), None));
        let restored = FuzzyDate::from_columns(y, m, d).unwrap();
        assert_eq!(date, restored);

        // Year only
        let date = FuzzyDate::Year {
            year: Year::new(1991).unwrap(),
        };
        let (y, m, d) = date.to_columns();
        assert_eq!((y, m, d), (1991, None, None));
        let restored = FuzzyDate::from_columns(y, m, d).unwrap();
        assert_eq!(date, restored);

        // Invalid: day without month
        let result = FuzzyDate::from_columns(1991, None, Some(15));
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_tuple() {
        // Full date
        let date: FuzzyDate = (1991, Some(8), Some(15)).try_into().unwrap();
        assert_eq!(date.year(), 1991);
        assert_eq!(date.month(), Some(8));
        assert_eq!(date.day(), Some(15));

        // Month year
        let date: FuzzyDate = (1991, Some(8), None).try_into().unwrap();
        assert_eq!(date.year(), 1991);
        assert_eq!(date.month(), Some(8));
        assert_eq!(date.day(), None);

        // Year only
        let date: FuzzyDate = (1991, None, None).try_into().unwrap();
        assert_eq!(date.year(), 1991);
        assert_eq!(date.month(), None);
        assert_eq!(date.day(), None);
    }

    #[test]
    fn test_century_non_leap_year() {
        // 1900 is not a leap year (divisible by 100 but not 400)
        let result = "02/29/1900".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidDay { .. })));

        // 2000 is a leap year (divisible by 400)
        let result = "02/29/2000".parse::<FuzzyDate>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_bad_tokens() {
        // Non-numeric year
        let result = "199A".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidFormat(_))));

        // Non-numeric month
        let result = "02/XX/2020".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidFormat(_))));

        // Non-numeric day
        let result = "1991-08-XX".parse::<FuzzyDate>();
        assert!(matches!(result, Err(ParseError::InvalidFormat(_))));
    }

    #[test]
    fn test_ordering_across_boundaries() {
        let jan31 = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(1).unwrap(),
            day: Day::new(31, 1991, 1).unwrap(),
        };

        let feb = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(2).unwrap(),
        };

        let feb01 = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(2).unwrap(),
            day: Day::new(1, 1991, 2).unwrap(),
        };

        assert!(jan31 < feb);
        assert!(jan31 < feb01);
        assert!(feb < feb01); // MonthYear comes before Full with same lower_bound
    }

    #[test]
    fn test_serde_string_format() {
        // Year only
        let date = FuzzyDate::Year {
            year: Year::new(1991).unwrap(),
        };
        let json = serde_json::to_string(&date).unwrap();
        assert_eq!(json, r#""1991""#);
        let parsed: FuzzyDate = serde_json::from_str(&json).unwrap();
        assert_eq!(date, parsed);

        // Month and year
        let date = FuzzyDate::Month {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
        };
        let json = serde_json::to_string(&date).unwrap();
        assert_eq!(json, r#""1991-08""#);
        let parsed: FuzzyDate = serde_json::from_str(&json).unwrap();
        assert_eq!(date, parsed);

        // Full date
        let date = FuzzyDate::Day {
            year: Year::new(1991).unwrap(),
            month: Month::new(8).unwrap(),
            day: Day::new(15, 1991, 8).unwrap(),
        };
        let json = serde_json::to_string(&date).unwrap();
        assert_eq!(json, r#""1991-08-15""#);
        let parsed: FuzzyDate = serde_json::from_str(&json).unwrap();
        assert_eq!(date, parsed);
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_YEAR, 9999);
    }

    #[test]
    fn test_serde_validation() {
        // Invalid month (13) should be rejected
        let json = r#""2024-13""#;
        let result: Result<FuzzyDate, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Invalid day (32) should be rejected
        let json = r#""2024-01-32""#;
        let result: Result<FuzzyDate, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Invalid day for February (30) should be rejected
        let json = r#""2024-02-30""#;
        let result: Result<FuzzyDate, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Invalid year (10000) should be rejected
        let json = r#""10000""#;
        let result: Result<FuzzyDate, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Valid values should succeed
        let json = r#""2024-12""#;
        let result: Result<FuzzyDate, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let json = r#""2024-02-29""#;
        let result: Result<FuzzyDate, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_too_many_date_separators() {
        // Too many hyphens in ISO format
        let result = "2000-01-15-23".parse::<FuzzyDate>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Too many - separators"));

        // Too many slashes in month-first format
        let result = "01/15/2000/extra".parse::<FuzzyDate>();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Too many / separators"));
    }
}
