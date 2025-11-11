use crate::prelude::*;
use crate::{FuzzyDate, ParseError, RANGE_SEPARATOR};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

/// Represents a range between two fuzzy dates (inclusive).
/// The start date must be less than or equal to the end date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display(fmt = "{start}/{end}")]
pub struct FuzzyDateRange {
    start: FuzzyDate,
    end: FuzzyDate,
}

/// Error type for date range operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RangeError {
    /// Start date is after end date.
    #[error("Invalid date range: start ({start}) is after end ({end})")]
    InvalidRange { start: FuzzyDate, end: FuzzyDate },

    /// Error parsing date component.
    #[error(transparent)]
    ParseError(#[from] ParseError),

    /// Invalid range format.
    #[error("Invalid range format: {0}")]
    InvalidFormat(String),
}

impl FuzzyDateRange {
    /// Creates a new date range with validation.
    /// Returns an error if start > end.
    pub fn new(start: FuzzyDate, end: FuzzyDate) -> Result<Self, RangeError> {
        if start > end {
            return Err(RangeError::InvalidRange { start, end });
        }
        Ok(Self { start, end })
    }

    /// Returns the start date of the range
    pub fn start(&self) -> FuzzyDate {
        self.start
    }

    /// Returns the end date of the range
    pub fn end(&self) -> FuzzyDate {
        self.end
    }

    /// Returns both start and end dates as a tuple
    pub fn dates(&self) -> (FuzzyDate, FuzzyDate) {
        (self.start, self.end)
    }

    /// Checks if the range contains a given date
    /// Uses concrete bounds comparison to handle mixed-precision dates correctly.
    pub fn contains(&self, date: &FuzzyDate) -> bool {
        let date_lower = date.lower_bound();
        let date_upper = date.upper_bound_inclusive();
        let range_lower = self.start.lower_bound();
        let range_upper = self.end.upper_bound_inclusive();

        // Date is contained if its bounds fall within the range's bounds
        range_lower <= date_lower && date_upper <= range_upper
    }

    /// Checks if this range overlaps with another range
    /// Uses concrete bounds comparison to handle mixed-precision ranges correctly.
    pub fn overlaps(&self, other: &FuzzyDateRange) -> bool {
        let self_lower = self.start.lower_bound();
        let self_upper = self.end.upper_bound_inclusive();
        let other_lower = other.start.lower_bound();
        let other_upper = other.end.upper_bound_inclusive();

        // Ranges overlap if they have any concrete dates in common
        self_lower <= other_upper && other_lower <= self_upper
    }

    /// Checks if this range is completely contained within another range
    /// Uses concrete bounds comparison to handle mixed-precision ranges correctly.
    pub fn is_within(&self, other: &FuzzyDateRange) -> bool {
        let self_lower = self.start.lower_bound();
        let self_upper = self.end.upper_bound_inclusive();
        let other_lower = other.start.lower_bound();
        let other_upper = other.end.upper_bound_inclusive();

        // Self is within other if its bounds fall within other's bounds
        other_lower <= self_lower && self_upper <= other_upper
    }

    /// Returns the earliest concrete date represented by this range.
    /// This is the `lower_bound` of the start date.
    pub fn lower_bound(&self) -> (u16, u8, u8) {
        self.start.lower_bound()
    }

    /// Returns the latest concrete date represented by this range (inclusive).
    /// This is the `upper_bound_inclusive` of the end date.
    pub fn upper_bound_inclusive(&self) -> (u16, u8, u8) {
        self.end.upper_bound_inclusive()
    }

    /// Returns the exclusive upper bound of this range.
    /// Returns None if it would overflow `MAX_YEAR` limit.
    pub fn upper_bound_exclusive(&self) -> Option<(u16, u8, u8)> {
        self.end.upper_bound_exclusive()
    }

    /// Converts to database columns: (`start_year`, `start_month`, `start_day`, `end_year`, `end_month`, `end_day`)
    pub fn to_columns(&self) -> (u16, Option<u8>, Option<u8>, u16, Option<u8>, Option<u8>) {
        let (sy, sm, sd) = self.start.to_columns();
        let (ey, em, ed) = self.end.to_columns();
        (sy, sm, sd, ey, em, ed)
    }

    /// Creates from database columns: (`start_year`, `start_month`, `start_day`, `end_year`, `end_month`, `end_day`)
    pub fn from_columns(
        start_year: u16,
        start_month: Option<u8>,
        start_day: Option<u8>,
        end_year: u16,
        end_month: Option<u8>,
        end_day: Option<u8>,
    ) -> Result<Self, RangeError> {
        let start = FuzzyDate::from_columns(start_year, start_month, start_day)?;
        let end = FuzzyDate::from_columns(end_year, end_month, end_day)?;
        Self::new(start, end)
    }
}

impl FromStr for FuzzyDateRange {
    type Err = RangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();

        // ISO 8601 extended format: use RANGE_SEPARATOR to separate start/end
        let separator_count = trimmed.matches(RANGE_SEPARATOR).count();

        match separator_count {
            0 => Err(RangeError::InvalidFormat(format!(
                "No range separator found (expected '{RANGE_SEPARATOR}'): {s}"
            ))),
            1 => {
                // SAFETY: We just verified separator_count == 1, so find() must succeed
                let pos = trimmed.find(RANGE_SEPARATOR).ok_or_else(|| {
                    RangeError::InvalidFormat(format!(
                        "Separator '{RANGE_SEPARATOR}' not found despite count == 1"
                    ))
                })?;
                let start_str = trimmed[..pos].trim();
                let end_str = trimmed[pos + 1..].trim();

                let start = start_str.parse::<FuzzyDate>()?;
                let end = end_str.parse::<FuzzyDate>()?;

                Self::new(start, end)
            }
            _ => Err(RangeError::InvalidFormat(format!(
                "Too many '{RANGE_SEPARATOR}' separators: expected 1, found {separator_count}"
            ))),
        }
    }
}

impl PartialOrd for FuzzyDateRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FuzzyDateRange {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare start dates first, then end dates
        match self.start.cmp(&other.start) {
            Ordering::Equal => self.end.cmp(&other.end),
            ord => ord,
        }
    }
}

impl Serialize for FuzzyDateRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for FuzzyDateRange {
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
    use crate::{Day, Month, Year};

    #[test]
    fn test_new_range_cases() {
        struct TestCase {
            start_year: u16,
            end_year: u16,
            should_succeed: bool,
            description: &'static str,
        }

        let cases = [
            TestCase {
                start_year: 1990,
                end_year: 2000,
                should_succeed: true,
                description: "valid range (start < end)",
            },
            TestCase {
                start_year: 2000,
                end_year: 1990,
                should_succeed: false,
                description: "invalid range (start > end)",
            },
            TestCase {
                start_year: 2000,
                end_year: 2000,
                should_succeed: true,
                description: "equal dates (start == end)",
            },
        ];

        for case in &cases {
            let start = FuzzyDate::new_year(Year::new(case.start_year).unwrap()).unwrap();
            let end = FuzzyDate::new_year(Year::new(case.end_year).unwrap()).unwrap();
            let range = FuzzyDateRange::new(start, end);

            if case.should_succeed {
                assert!(range.is_ok(), "Expected success for: {}", case.description);
            } else {
                assert!(range.is_err(), "Expected failure for: {}", case.description);
            }
        }
    }

    #[test]
    fn test_accessors() {
        let start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);
        assert_eq!(range.dates(), (start, end));
    }

    #[test]
    fn test_contains() {
        let start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        let mid = FuzzyDate::new_year(Year::new(1995).unwrap()).unwrap();
        let before = FuzzyDate::new_year(Year::new(1980).unwrap()).unwrap();
        let after = FuzzyDate::new_year(Year::new(2010).unwrap()).unwrap();

        assert!(range.contains(&start));
        assert!(range.contains(&end));
        assert!(range.contains(&mid));
        assert!(!range.contains(&before));
        assert!(!range.contains(&after));
    }

    #[test]
    fn test_overlaps() {
        let range1_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range1_end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let range1 = FuzzyDateRange::new(range1_start, range1_end).unwrap();

        // Overlapping range
        let range2_start = FuzzyDate::new_year(Year::new(1995).unwrap()).unwrap();
        let range2_end = FuzzyDate::new_year(Year::new(2005).unwrap()).unwrap();
        let range2 = FuzzyDateRange::new(range2_start, range2_end).unwrap();

        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));

        // Non-overlapping range
        let range3_start = FuzzyDate::new_year(Year::new(2010).unwrap()).unwrap();
        let range3_end = FuzzyDate::new_year(Year::new(2020).unwrap()).unwrap();
        let range3 = FuzzyDateRange::new(range3_start, range3_end).unwrap();

        assert!(!range1.overlaps(&range3));
        assert!(!range3.overlaps(&range1));
    }

    #[test]
    fn test_is_within() {
        let outer_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let outer_end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let outer = FuzzyDateRange::new(outer_start, outer_end).unwrap();

        let inner_start = FuzzyDate::new_year(Year::new(1995).unwrap()).unwrap();
        let inner_end = FuzzyDate::new_year(Year::new(1998).unwrap()).unwrap();
        let inner = FuzzyDateRange::new(inner_start, inner_end).unwrap();

        assert!(inner.is_within(&outer));
        assert!(!outer.is_within(&inner));
    }

    #[test]
    fn test_bounds() {
        let start = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(6).unwrap(),
            Day::new(15, 1990, 6).unwrap(),
        )
        .unwrap();
        let end = FuzzyDate::new_day(
            Year::new(2000).unwrap(),
            Month::new(12).unwrap(),
            Day::new(31, 2000, 12).unwrap(),
        )
        .unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        assert_eq!(range.lower_bound(), (1990, 6, 15));
        assert_eq!(range.upper_bound_inclusive(), (2000, 12, 31));
        assert_eq!(range.upper_bound_exclusive(), Some((2001, 1, 1)));
    }

    #[test]
    fn test_display() {
        let start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        assert_eq!(range.to_string(), "1990/2000");
    }

    #[test]
    fn test_from_str_with_slash() {
        let range = "1990/2000".parse::<FuzzyDateRange>().unwrap();
        assert_eq!(range.start().year(), 1990);
        assert_eq!(range.end().year(), 2000);
    }

    #[test]
    fn test_from_str_with_month_precision() {
        let range = "1990-01/2000-12".parse::<FuzzyDateRange>().unwrap();
        assert_eq!(range.start().year(), 1990);
        assert_eq!(range.start().month(), Some(1));
        assert_eq!(range.end().year(), 2000);
        assert_eq!(range.end().month(), Some(12));
    }

    #[test]
    fn test_from_str_with_day_precision() {
        let range = "1990-01-15/2000-12-31".parse::<FuzzyDateRange>().unwrap();
        assert_eq!(range.start().year(), 1990);
        assert_eq!(range.start().month(), Some(1));
        assert_eq!(range.start().day(), Some(15));
        assert_eq!(range.end().year(), 2000);
        assert_eq!(range.end().month(), Some(12));
        assert_eq!(range.end().day(), Some(31));
    }

    #[test]
    fn test_from_str_invalid_order() {
        let result = "2000/1990".parse::<FuzzyDateRange>();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_no_delimiter() {
        let result = "19902000".parse::<FuzzyDateRange>();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_rejects_other_delimiters() {
        // Should reject " to " delimiter
        let result = "1990 to 2000".parse::<FuzzyDateRange>();
        assert!(result.is_err());

        // Should reject " - " delimiter
        let result = "1990 - 2000".parse::<FuzzyDateRange>();
        assert!(result.is_err());

        // Should reject ".." delimiter
        let result = "1990..2000".parse::<FuzzyDateRange>();
        assert!(result.is_err());
    }

    #[test]
    fn test_ordering() {
        let range1_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range1_end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let range1 = FuzzyDateRange::new(range1_start, range1_end).unwrap();

        let range2_start = FuzzyDate::new_year(Year::new(1995).unwrap()).unwrap();
        let range2_end = FuzzyDate::new_year(Year::new(2005).unwrap()).unwrap();
        let range2 = FuzzyDateRange::new(range2_start, range2_end).unwrap();

        assert!(range1 < range2);
        assert!(range2 > range1);
    }

    #[test]
    fn test_ordering_same_start() {
        let range1_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range1_end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let range1 = FuzzyDateRange::new(range1_start, range1_end).unwrap();

        let range2_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range2_end = FuzzyDate::new_year(Year::new(2005).unwrap()).unwrap();
        let range2 = FuzzyDateRange::new(range2_start, range2_end).unwrap();

        assert!(range1 < range2);
    }

    #[test]
    fn test_to_columns_and_from_columns() {
        let start = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(6).unwrap(),
            Day::new(15, 1990, 6).unwrap(),
        )
        .unwrap();
        let end = FuzzyDate::new_day(
            Year::new(2000).unwrap(),
            Month::new(12).unwrap(),
            Day::new(31, 2000, 12).unwrap(),
        )
        .unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        let (sy, sm, sd, ey, em, ed) = range.to_columns();
        assert_eq!((sy, sm, sd), (1990, Some(6), Some(15)));
        assert_eq!((ey, em, ed), (2000, Some(12), Some(31)));

        let restored = FuzzyDateRange::from_columns(sy, sm, sd, ey, em, ed).unwrap();
        assert_eq!(range, restored);
    }

    #[test]
    fn test_mixed_precision_range() {
        let start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let end = FuzzyDate::new_day(
            Year::new(2000).unwrap(),
            Month::new(12).unwrap(),
            Day::new(31, 2000, 12).unwrap(),
        )
        .unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        assert_eq!(range.lower_bound(), (1990, 1, 1));
        assert_eq!(range.upper_bound_inclusive(), (2000, 12, 31));
    }

    #[test]
    fn test_from_str_mixed_precision_month_to_year() {
        let range = "1991-08/2025".parse::<FuzzyDateRange>().unwrap();
        assert_eq!(range.start().year(), 1991);
        assert_eq!(range.start().month(), Some(8));
        assert_eq!(range.start().day(), None);
        assert_eq!(range.end().year(), 2025);
        assert_eq!(range.end().month(), None);
        assert_eq!(range.end().day(), None);
    }

    #[test]
    fn test_from_str_mixed_precision_year_to_day() {
        let range = "1990/2025-12-31".parse::<FuzzyDateRange>().unwrap();
        assert_eq!(range.start().year(), 1990);
        assert_eq!(range.start().month(), None);
        assert_eq!(range.end().year(), 2025);
        assert_eq!(range.end().month(), Some(12));
        assert_eq!(range.end().day(), Some(31));
    }

    #[test]
    fn test_from_str_mixed_precision_day_to_month() {
        let range = "1990-01-15/2025-12".parse::<FuzzyDateRange>().unwrap();
        assert_eq!(range.start().year(), 1990);
        assert_eq!(range.start().month(), Some(1));
        assert_eq!(range.start().day(), Some(15));
        assert_eq!(range.end().year(), 2025);
        assert_eq!(range.end().month(), Some(12));
        assert_eq!(range.end().day(), None);
    }

    #[test]
    fn test_display_mixed_precision() {
        let start = FuzzyDate::new_month(Year::new(1991).unwrap(), Month::new(8).unwrap()).unwrap();
        let end = FuzzyDate::new_year(Year::new(2025).unwrap()).unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        assert_eq!(range.to_string(), "1991-08/2025");
    }

    #[test]
    fn test_serde_string_format() {
        let start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let end = FuzzyDate::new_year(Year::new(2000).unwrap()).unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        let json = serde_json::to_string(&range).unwrap();
        // Should be a JSON string, not an object
        assert_eq!(json, r#""1990/2000""#);

        let parsed: FuzzyDateRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range, parsed);
    }

    #[test]
    fn test_serde_mixed_precision() {
        let start = FuzzyDate::new_month(Year::new(1991).unwrap(), Month::new(8).unwrap()).unwrap();
        let end = FuzzyDate::new_year(Year::new(2025).unwrap()).unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(json, r#""1991-08/2025""#);

        let parsed: FuzzyDateRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range, parsed);
    }

    #[test]
    fn test_serde_full_precision() {
        let start = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(6).unwrap(),
            Day::new(15, 1990, 6).unwrap(),
        )
        .unwrap();
        let end = FuzzyDate::new_day(
            Year::new(2000).unwrap(),
            Month::new(12).unwrap(),
            Day::new(31, 2000, 12).unwrap(),
        )
        .unwrap();
        let range = FuzzyDateRange::new(start, end).unwrap();

        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(json, r#""1990-06-15/2000-12-31""#);

        let parsed: FuzzyDateRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range, parsed);
    }

    #[test]
    fn test_too_many_range_separators() {
        // Too many '/' separators
        let result = "2000/2001/2002".parse::<FuzzyDateRange>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Too many '/' separators"));
        assert!(err.to_string().contains("expected 1, found 2"));
    }

    #[test]
    fn test_no_range_separator() {
        // Missing '/' separator
        let result = "20002001".parse::<FuzzyDateRange>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No range separator found"));
    }

    #[test]
    fn test_contains_mixed_precision_year_contains_month() {
        // Year range should contain month within that year
        let range_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range_end = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range = FuzzyDateRange::new(range_start, range_end).unwrap();

        let date = FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        assert!(range.contains(&date), "1990/1990 should contain 1990-06");
    }

    #[test]
    fn test_contains_mixed_precision_year_contains_day() {
        // Year range should contain day within that year
        let range_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range_end = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range = FuzzyDateRange::new(range_start, range_end).unwrap();

        let date = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(6).unwrap(),
            Day::new(15, 1990, 6).unwrap(),
        )
        .unwrap();
        assert!(range.contains(&date), "1990/1990 should contain 1990-06-15");
    }

    #[test]
    fn test_contains_mixed_precision_month_contains_day() {
        // Month range should contain day within that month
        let range_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let range_end =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let range = FuzzyDateRange::new(range_start, range_end).unwrap();

        let date = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(6).unwrap(),
            Day::new(15, 1990, 6).unwrap(),
        )
        .unwrap();
        assert!(
            range.contains(&date),
            "1990-06/1990-06 should contain 1990-06-15"
        );
    }

    #[test]
    fn test_contains_mixed_precision_month_not_contains_different_month() {
        // Month range should not contain day from different month
        let range_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let range_end =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let range = FuzzyDateRange::new(range_start, range_end).unwrap();

        let date = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(7).unwrap(),
            Day::new(1, 1990, 7).unwrap(),
        )
        .unwrap();
        assert!(
            !range.contains(&date),
            "1990-06/1990-06 should not contain 1990-07-01"
        );
    }

    #[test]
    fn test_overlaps_mixed_precision_year_overlaps_month() {
        // Year range overlaps with month in that year
        let range1_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range1_end = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range1 = FuzzyDateRange::new(range1_start, range1_end).unwrap();

        let range2_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let range2_end =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(8).unwrap()).unwrap();
        let range2 = FuzzyDateRange::new(range2_start, range2_end).unwrap();

        assert!(
            range1.overlaps(&range2),
            "1990/1990 should overlap 1990-06/1990-08"
        );
        assert!(
            range2.overlaps(&range1),
            "1990-06/1990-08 should overlap 1990/1990"
        );
    }

    #[test]
    fn test_overlaps_mixed_precision_partial_year_overlap() {
        // Year range partially overlaps with range extending beyond
        let range1_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range1_end = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let range1 = FuzzyDateRange::new(range1_start, range1_end).unwrap();

        let range2_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let range2_end =
            FuzzyDate::new_month(Year::new(1991).unwrap(), Month::new(2).unwrap()).unwrap();
        let range2 = FuzzyDateRange::new(range2_start, range2_end).unwrap();

        assert!(
            range1.overlaps(&range2),
            "1990/1990 should overlap 1990-06/1991-02"
        );
        assert!(
            range2.overlaps(&range1),
            "1990-06/1991-02 should overlap 1990/1990"
        );
    }

    #[test]
    fn test_is_within_mixed_precision_month_within_year() {
        // Month range is within year range
        let outer_start = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let outer_end = FuzzyDate::new_year(Year::new(1990).unwrap()).unwrap();
        let outer = FuzzyDateRange::new(outer_start, outer_end).unwrap();

        let inner_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let inner_end =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(8).unwrap()).unwrap();
        let inner = FuzzyDateRange::new(inner_start, inner_end).unwrap();

        assert!(
            inner.is_within(&outer),
            "1990-06/1990-08 should be within 1990/1990"
        );
        assert!(
            !outer.is_within(&inner),
            "1990/1990 should not be within 1990-06/1990-08"
        );
    }

    #[test]
    fn test_is_within_mixed_precision_day_within_month() {
        // Day range is within month range
        let outer_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let outer_end =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let outer = FuzzyDateRange::new(outer_start, outer_end).unwrap();

        let inner_start = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(6).unwrap(),
            Day::new(10, 1990, 6).unwrap(),
        )
        .unwrap();
        let inner_end = FuzzyDate::new_day(
            Year::new(1990).unwrap(),
            Month::new(6).unwrap(),
            Day::new(20, 1990, 6).unwrap(),
        )
        .unwrap();
        let inner = FuzzyDateRange::new(inner_start, inner_end).unwrap();

        assert!(
            inner.is_within(&outer),
            "1990-06-10/1990-06-20 should be within 1990-06/1990-06"
        );
    }

    #[test]
    fn test_is_within_mixed_precision_extends_beyond() {
        // Range extending beyond should not be within
        let outer_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(6).unwrap()).unwrap();
        let outer_end =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(8).unwrap()).unwrap();
        let outer = FuzzyDateRange::new(outer_start, outer_end).unwrap();

        let inner_start =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(7).unwrap()).unwrap();
        let inner_end =
            FuzzyDate::new_month(Year::new(1990).unwrap(), Month::new(9).unwrap()).unwrap();
        let inner = FuzzyDateRange::new(inner_start, inner_end).unwrap();

        assert!(
            !inner.is_within(&outer),
            "1990-07/1990-09 should not be within 1990-06/1990-08"
        );
    }
}
