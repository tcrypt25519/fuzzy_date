/// Maximum valid year (inclusive)
pub const MAX_YEAR: u16 = 9999;

/// Maximum valid month (December)
pub const MAX_MONTH: u8 = 12;

/// First day of month, used for lower bounds
pub const MIN_DAY: u8 = 1;

/// Month number for January
pub const JANUARY: u8 = 1;
/// Month number for February
pub const FEBRUARY: u8 = 2;
/// Month number for December
pub const DECEMBER: u8 = 12;

/// Days in February for leap years
pub const FEBRUARY_DAYS_LEAP: u8 = 29;

/// Maximum days in each month (index 0 is unused, months are 1-indexed)
/// February shows 28 days (non-leap year default)
pub const DAYS_IN_MONTH: [u8; 13] = [
    0,  // index 0 unused (months are 1-indexed)
    31, // January
    28, // February (non-leap, adjusted by is_leap_year check)
    31, // March
    30, // April
    31, // May
    30, // June
    31, // July
    31, // August
    30, // September
    31, // October
    30, // November
    31, // December
];

/// Leap year occurs every 4 years
pub(crate) const LEAP_YEAR_CYCLE: u16 = 4;
/// Century years are not leap years unless...
pub(crate) const CENTURY_CYCLE: u16 = 100;
/// ...they are divisible by 400 (Gregorian calendar correction)
pub(crate) const GREGORIAN_CYCLE: u16 = 400;

/// Date component separator (ISO 8601 format)
pub const DATE_SEPARATOR: char = '-';
/// Range separator (ISO 8601 extended format)
pub const RANGE_SEPARATOR: char = '/';
/// Month-first format separator (legacy US format)
pub const MONTH_FIRST_SEPARATOR: char = '/';
