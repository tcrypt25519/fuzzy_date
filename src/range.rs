use std::{cmp::Ordering, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{FuzzyDate, ParseError, RANGE_SEPARATOR, prelude::*};

/// Represents a range between two fuzzy dates (inclusive).
/// The start date must be less than or equal to the end date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
#[display(fmt = "{start}/{end}")]
pub struct FuzzyDateRange {
    start: FuzzyDate,
    end:   FuzzyDate,
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
    ///
    /// # Errors
    /// Returns `RangeError::InvalidRange` if start > end.
    pub fn new(start: FuzzyDate, end: FuzzyDate) -> Result<Self, RangeError> {
        if start > end {
            return Err(RangeError::InvalidRange { start, end });
        }
        Ok(Self { start, end })
    }

    /// Returns the start date of the range
    pub const fn start(&self) -> FuzzyDate {
        self.start
    }

    /// Returns the end date of the range
    pub const fn end(&self) -> FuzzyDate {
        self.end
    }

    /// Returns both start and end dates as a tuple
    pub const fn dates(&self) -> (FuzzyDate, FuzzyDate) {
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
    pub fn overlaps(&self, other: &Self) -> bool {
        let self_lower = self.start.lower_bound();
        let self_upper = self.end.upper_bound_inclusive();
        let other_lower = other.start.lower_bound();
        let other_upper = other.end.upper_bound_inclusive();

        // Ranges overlap if they have any concrete dates in common
        self_lower <= other_upper && other_lower <= self_upper
    }

    /// Checks if this range is completely contained within another range
    /// Uses concrete bounds comparison to handle mixed-precision ranges correctly.
    pub fn is_within(&self, other: &Self) -> bool {
        let self_lower = self.start.lower_bound();
        let self_upper = self.end.upper_bound_inclusive();
        let other_lower = other.start.lower_bound();
        let other_upper = other.end.upper_bound_inclusive();

        // Self is within other if its bounds fall within other's bounds
        other_lower <= self_lower && self_upper <= other_upper
    }

    /// Returns the earliest concrete date represented by this range.
    /// This is the `lower_bound` of the start date.
    pub const fn lower_bound(&self) -> (u16, u8, u8) {
        self.start.lower_bound()
    }

    /// Returns the latest concrete date represented by this range (inclusive).
    /// This is the `upper_bound_inclusive` of the end date.
    pub const fn upper_bound_inclusive(&self) -> (u16, u8, u8) {
        self.end.upper_bound_inclusive()
    }

    /// Returns the exclusive upper bound of this range.
    /// Returns None if it would overflow `MAX_YEAR` limit.
    pub fn upper_bound_exclusive(&self) -> Option<(u16, u8, u8)> {
        self.end.upper_bound_exclusive()
    }

    /// Converts to database columns: (`start_year`, `start_month`, `start_day`, `end_year`, `end_month`, `end_day`)
    pub const fn to_columns(&self) -> (u16, Option<u8>, Option<u8>, u16, Option<u8>, Option<u8>) {
        let (sy, sm, sd) = self.start.to_columns();
        let (ey, em, ed) = self.end.to_columns();
        (sy, sm, sd, ey, em, ed)
    }

    /// Creates from database columns: (`start_year`, `start_month`, `start_day`, `end_year`, `end_month`, `end_day`)
    ///
    /// # Errors
    /// Returns `RangeError` if the dates are invalid or start > end.
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
                    RangeError::InvalidFormat(format!("Separator '{RANGE_SEPARATOR}' not found despite count == 1"))
                })?;
                let start_str = trimmed[..pos].trim();
                let end_str = trimmed[pos + 1..].trim();

                let start = start_str.parse::<FuzzyDate>()?;
                let end = end_str.parse::<FuzzyDate>()?;

                Self::new(start, end)
            },
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
    use crate::test_utils::{day, fuzzy_day, fuzzy_month, fuzzy_year, month, year};

    #[test]
    fn test_new_range_cases() {
        struct TestCase {
            start_year:     u16,
            end_year:       u16,
            should_succeed: bool,
            description:    &'static str,
        }

        let cases = [
            TestCase {
                start_year:     1990,
                end_year:       2000,
                should_succeed: true,
                description:    "valid range (start < end)",
            },
            TestCase {
                start_year:     2000,
                end_year:       1990,
                should_succeed: false,
                description:    "invalid range (start > end)",
            },
            TestCase {
                start_year:     2000,
                end_year:       2000,
                should_succeed: true,
                description:    "equal dates (start == end)",
            },
        ];

        for case in &cases {
            let start = fuzzy_year(case.start_year);
            let end = fuzzy_year(case.end_year);
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
        let start = fuzzy_year(1990);
        let end = fuzzy_year(2000);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for accessor test");

        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);
        assert_eq!(range.dates(), (start, end));
    }

    #[test]
    fn test_contains() {
        let start = fuzzy_year(1990);
        let end = fuzzy_year(2000);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for contains test");

        let mid = fuzzy_year(1995);
        let before = fuzzy_year(1980);
        let after = fuzzy_year(2010);

        assert!(range.contains(&start));
        assert!(range.contains(&end));
        assert!(range.contains(&mid));
        assert!(!range.contains(&before));
        assert!(!range.contains(&after));
    }

    #[test]
    fn test_overlaps() {
        let range1_start = fuzzy_year(1990);
        let range1_end = fuzzy_year(2000);
        let range1 =
            FuzzyDateRange::new(range1_start, range1_end).expect("failed to construct first range for overlaps test");

        // Overlapping range
        let range2_start = fuzzy_year(1995);
        let range2_end = fuzzy_year(2005);
        let range2 = FuzzyDateRange::new(range2_start, range2_end)
            .expect("failed to construct overlapping range for overlaps test");

        assert!(range1.overlaps(&range2));
        assert!(range2.overlaps(&range1));

        // Non-overlapping range
        let range3_start = fuzzy_year(2010);
        let range3_end = fuzzy_year(2020);
        let range3 = FuzzyDateRange::new(range3_start, range3_end)
            .expect("failed to construct non-overlapping range for overlaps test");

        assert!(!range1.overlaps(&range3));
        assert!(!range3.overlaps(&range1));
    }

    #[test]
    fn test_is_within() {
        let outer_start = fuzzy_year(1990);
        let outer_end = fuzzy_year(2000);
        let outer =
            FuzzyDateRange::new(outer_start, outer_end).expect("failed to construct outer range for containment test");

        let inner_start = fuzzy_year(1995);
        let inner_end = fuzzy_year(1998);
        let inner =
            FuzzyDateRange::new(inner_start, inner_end).expect("failed to construct inner range for containment test");

        assert!(inner.is_within(&outer));
        assert!(!outer.is_within(&inner));
    }

    #[test]
    fn test_bounds() {
        let start = fuzzy_day(1990, 6, 15);
        let end = fuzzy_day(2000, 12, 31);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for bounds test");

        assert_eq!(range.lower_bound(), (1990, 6, 15));
        assert_eq!(range.upper_bound_inclusive(), (2000, 12, 31));
        assert_eq!(range.upper_bound_exclusive(), Some((2001, 1, 1)));
    }

    #[test]
    fn test_display() {
        let start = fuzzy_year(1990);
        let end = fuzzy_year(2000);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for display test");

        assert_eq!(range.to_string(), "1990/2000");
    }

    #[test]
    fn test_from_str_with_slash() {
        let range = "1990/2000".parse::<FuzzyDateRange>().expect("failed to parse range with slash");
        assert_eq!(range.start().year(), year(1990));
        assert_eq!(range.end().year(), year(2000));
    }

    #[test]
    fn test_from_str_with_month_precision() {
        let range = "1990-01/2000-12"
            .parse::<FuzzyDateRange>()
            .expect("failed to parse month-precision range");
        assert_eq!(range.start().year(), year(1990));
        assert_eq!(range.start().month(), Some(month(1)));
        assert_eq!(range.end().year(), year(2000));
        assert_eq!(range.end().month(), Some(month(12)));
    }

    #[test]
    fn test_from_str_with_day_precision() {
        let range = "1990-01-15/2000-12-31"
            .parse::<FuzzyDateRange>()
            .expect("failed to parse day-precision range");
        assert_eq!(range.start().year(), year(1990));
        assert_eq!(range.start().month(), Some(month(1)));
        assert_eq!(range.start().day(), Some(day(15, 1990, 1)));
        assert_eq!(range.end().year(), year(2000));
        assert_eq!(range.end().month(), Some(month(12)));
        assert_eq!(range.end().day(), Some(day(31, 2000, 12)));
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
        let range1_start = fuzzy_year(1990);
        let range1_end = fuzzy_year(2000);
        let range1 =
            FuzzyDateRange::new(range1_start, range1_end).expect("failed to construct first range for ordering test");

        let range2_start = fuzzy_year(1995);
        let range2_end = fuzzy_year(2005);
        let range2 =
            FuzzyDateRange::new(range2_start, range2_end).expect("failed to construct second range for ordering test");

        assert!(range1 < range2);
        assert!(range2 > range1);
    }

    #[test]
    fn test_ordering_same_start() {
        let range1_start = fuzzy_year(1990);
        let range1_end = fuzzy_year(2000);
        let range1 = FuzzyDateRange::new(range1_start, range1_end)
            .expect("failed to construct first range for equal-start ordering test");

        let range2_start = fuzzy_year(1990);
        let range2_end = fuzzy_year(2005);
        let range2 = FuzzyDateRange::new(range2_start, range2_end)
            .expect("failed to construct second range for equal-start ordering test");

        assert!(range1 < range2);
    }

    #[test]
    fn test_to_columns_and_from_columns() {
        let start = fuzzy_day(1990, 6, 15);
        let end = fuzzy_day(2000, 12, 31);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for column conversion test");

        let (sy, sm, sd, ey, em, ed) = range.to_columns();
        assert_eq!((sy, sm, sd), (1990, Some(6), Some(15)));
        assert_eq!((ey, em, ed), (2000, Some(12), Some(31)));

        let restored =
            FuzzyDateRange::from_columns(sy, sm, sd, ey, em, ed).expect("failed to restore range from columns");
        assert_eq!(range, restored);
    }

    #[test]
    fn test_mixed_precision_range() {
        let start = fuzzy_year(1990);
        let end = fuzzy_day(2000, 12, 31);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct mixed-precision range");

        assert_eq!(range.lower_bound(), (1990, 1, 1));
        assert_eq!(range.upper_bound_inclusive(), (2000, 12, 31));
    }

    #[test]
    fn test_from_str_mixed_precision_month_to_year() {
        let range = "1991-08/2025"
            .parse::<FuzzyDateRange>()
            .expect("failed to parse month-to-year range");
        assert_eq!(range.start().year(), year(1991));
        assert_eq!(range.start().month(), Some(month(8)));
        assert_eq!(range.start().day(), None);
        assert_eq!(range.end().year(), year(2025));
        assert_eq!(range.end().month(), None);
        assert_eq!(range.end().day(), None);
    }

    #[test]
    fn test_from_str_mixed_precision_year_to_day() {
        let range = "1990/2025-12-31"
            .parse::<FuzzyDateRange>()
            .expect("failed to parse year-to-day range");
        assert_eq!(range.start().year(), year(1990));
        assert_eq!(range.start().month(), None);
        assert_eq!(range.end().year(), year(2025));
        assert_eq!(range.end().month(), Some(month(12)));
        assert_eq!(range.end().day(), Some(day(31, 2025, 12)));
    }

    #[test]
    fn test_from_str_mixed_precision_day_to_month() {
        let range = "1990-01-15/2025-12"
            .parse::<FuzzyDateRange>()
            .expect("failed to parse day-to-month range");
        assert_eq!(range.start().year(), year(1990));
        assert_eq!(range.start().month(), Some(month(1)));
        assert_eq!(range.start().day(), Some(day(15, 1990, 1)));
        assert_eq!(range.end().year(), year(2025));
        assert_eq!(range.end().month(), Some(month(12)));
        assert_eq!(range.end().day(), None);
    }

    #[test]
    fn test_display_mixed_precision() {
        let start = fuzzy_month(1991, 8);
        let end = fuzzy_year(2025);
        let range =
            FuzzyDateRange::new(start, end).expect("failed to construct range for mixed-precision display test");

        assert_eq!(range.to_string(), "1991-08/2025");
    }

    #[test]
    fn test_serde_string_format() {
        let start = fuzzy_year(1990);
        let end = fuzzy_year(2000);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for serde string test");

        let json = serde_json::to_string(&range).expect("failed to serialize range to JSON");
        // Should be a JSON string, not an object
        assert_eq!(json, r#""1990/2000""#);

        let parsed: FuzzyDateRange = serde_json::from_str(&json).expect("failed to deserialize range from JSON");
        assert_eq!(range, parsed);
    }

    #[test]
    fn test_serde_mixed_precision() {
        let start = fuzzy_month(1991, 8);
        let end = fuzzy_year(2025);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for serde mixed-precision test");

        let json = serde_json::to_string(&range).expect("failed to serialize mixed-precision range");
        assert_eq!(json, r#""1991-08/2025""#);

        let parsed: FuzzyDateRange =
            serde_json::from_str(&json).expect("failed to deserialize mixed-precision range from JSON");
        assert_eq!(range, parsed);
    }

    #[test]
    fn test_serde_full_precision() {
        let start = fuzzy_day(1990, 6, 15);
        let end = fuzzy_day(2000, 12, 31);
        let range = FuzzyDateRange::new(start, end).expect("failed to construct range for serde full-precision test");

        let json = serde_json::to_string(&range).expect("failed to serialize full-precision range");
        assert_eq!(json, r#""1990-06-15/2000-12-31""#);

        let parsed: FuzzyDateRange = serde_json::from_str(&json).expect("failed to deserialize full-precision range");
        assert_eq!(range, parsed);
    }

    #[test]
    fn test_too_many_range_separators() {
        // Too many '/' separators
        let result = "2000/2001/2002".parse::<FuzzyDateRange>();
        assert!(result.is_err());
        let err = result.expect_err("expected error for too many range separators");
        assert!(err.to_string().contains("Too many '/' separators"));
        assert!(err.to_string().contains("expected 1, found 2"));
    }

    #[test]
    fn test_no_range_separator() {
        // Missing '/' separator
        let result = "20002001".parse::<FuzzyDateRange>();
        assert!(result.is_err());
        let err = result.expect_err("expected error for missing range separator");
        assert!(err.to_string().contains("No range separator found"));
    }

    #[test]
    fn test_contains_mixed_precision_year_contains_month() {
        // Year range should contain month within that year
        let range_start = fuzzy_year(1990);
        let range_end = fuzzy_year(1990);
        let range =
            FuzzyDateRange::new(range_start, range_end).expect("failed to construct year range for contains test");

        let date = fuzzy_month(1990, 6);
        assert!(range.contains(&date), "1990/1990 should contain 1990-06");
    }

    #[test]
    fn test_contains_mixed_precision_year_contains_day() {
        // Year range should contain day within that year
        let range_start = fuzzy_year(1990);
        let range_end = fuzzy_year(1990);
        let range = FuzzyDateRange::new(range_start, range_end)
            .expect("failed to construct year range for day containment test");

        let date = fuzzy_day(1990, 6, 15);
        assert!(range.contains(&date), "1990/1990 should contain 1990-06-15");
    }

    #[test]
    fn test_contains_mixed_precision_month_contains_day() {
        // Month range should contain day within that month
        let range_start = fuzzy_month(1990, 6);
        let range_end = fuzzy_month(1990, 6);
        let range = FuzzyDateRange::new(range_start, range_end)
            .expect("failed to construct month range for day containment test");

        let date = fuzzy_day(1990, 6, 15);
        assert!(range.contains(&date), "1990-06/1990-06 should contain 1990-06-15");
    }

    #[test]
    fn test_contains_mixed_precision_month_not_contains_different_month() {
        // Month range should not contain day from different month
        let range_start = fuzzy_month(1990, 6);
        let range_end = fuzzy_month(1990, 6);
        let range = FuzzyDateRange::new(range_start, range_end)
            .expect("failed to construct month range for different-month containment test");

        let date = fuzzy_day(1990, 7, 1);
        assert!(!range.contains(&date), "1990-06/1990-06 should not contain 1990-07-01");
    }

    #[test]
    fn test_overlaps_mixed_precision_year_overlaps_month() {
        // Year range overlaps with month in that year
        let range1_start = fuzzy_year(1990);
        let range1_end = fuzzy_year(1990);
        let range1 =
            FuzzyDateRange::new(range1_start, range1_end).expect("failed to construct year range for overlaps test");

        let range2_start = fuzzy_month(1990, 6);
        let range2_end = fuzzy_month(1990, 8);
        let range2 =
            FuzzyDateRange::new(range2_start, range2_end).expect("failed to construct month range for overlaps test");

        assert!(range1.overlaps(&range2), "1990/1990 should overlap 1990-06/1990-08");
        assert!(range2.overlaps(&range1), "1990-06/1990-08 should overlap 1990/1990");
    }

    #[test]
    fn test_overlaps_mixed_precision_partial_year_overlap() {
        // Year range partially overlaps with range extending beyond
        let range1_start = fuzzy_year(1990);
        let range1_end = fuzzy_year(1990);
        let range1 = FuzzyDateRange::new(range1_start, range1_end)
            .expect("failed to construct first year range for partial overlap test");

        let range2_start = fuzzy_month(1990, 6);
        let range2_end = fuzzy_month(1991, 2);
        let range2 = FuzzyDateRange::new(range2_start, range2_end)
            .expect("failed to construct second range for partial overlap test");

        assert!(range1.overlaps(&range2), "1990/1990 should overlap 1990-06/1991-02");
        assert!(range2.overlaps(&range1), "1990-06/1991-02 should overlap 1990/1990");
    }

    #[test]
    fn test_is_within_mixed_precision_month_within_year() {
        // Month range is within year range
        let outer_start = fuzzy_year(1990);
        let outer_end = fuzzy_year(1990);
        let outer = FuzzyDateRange::new(outer_start, outer_end)
            .expect("failed to construct outer year range for month containment test");

        let inner_start = fuzzy_month(1990, 6);
        let inner_end = fuzzy_month(1990, 8);
        let inner = FuzzyDateRange::new(inner_start, inner_end)
            .expect("failed to construct inner month range for month containment test");

        assert!(inner.is_within(&outer), "1990-06/1990-08 should be within 1990/1990");
        assert!(!outer.is_within(&inner), "1990/1990 should not be within 1990-06/1990-08");
    }

    #[test]
    fn test_is_within_mixed_precision_day_within_month() {
        // Day range is within month range
        let outer_start = fuzzy_month(1990, 6);
        let outer_end = fuzzy_month(1990, 6);
        let outer = FuzzyDateRange::new(outer_start, outer_end)
            .expect("failed to construct outer month range for day containment test");

        let inner_start = fuzzy_day(1990, 6, 10);
        let inner_end = fuzzy_day(1990, 6, 20);
        let inner = FuzzyDateRange::new(inner_start, inner_end)
            .expect("failed to construct inner day range for day containment test");

        assert!(
            inner.is_within(&outer),
            "1990-06-10/1990-06-20 should be within 1990-06/1990-06"
        );
    }

    #[test]
    fn test_is_within_mixed_precision_extends_beyond() {
        // Range extending beyond should not be within
        let outer_start = fuzzy_month(1990, 6);
        let outer_end = fuzzy_month(1990, 8);
        let outer = FuzzyDateRange::new(outer_start, outer_end)
            .expect("failed to construct outer range for extension containment test");

        let inner_start = fuzzy_month(1990, 7);
        let inner_end = fuzzy_month(1990, 9);
        let inner = FuzzyDateRange::new(inner_start, inner_end)
            .expect("failed to construct inner range for extension containment test");

        assert!(!inner.is_within(&outer), "1990-07/1990-09 should not be within 1990-06/1990-08");
    }
}
