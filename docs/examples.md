# fuzzy_date examples

Full examples for `FuzzyDate` and `FuzzyDateRange`. For a high-level overview,
see the [README](../README.md).

---

## Parsing

`FuzzyDate` parses from strings using `FromStr`. The precision is inferred from
the format — no configuration needed.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDate;

let year:  FuzzyDate = "2026".parse()?;
let month: FuzzyDate = "2026-02".parse()?;
let day:   FuzzyDate = "2026-02-13".parse()?;

// US month-first formats
let month_us: FuzzyDate = "02/2026".parse()?;
let day_us:   FuzzyDate = "02/13/2026".parse()?;

// Whitespace is trimmed
let trimmed: FuzzyDate = " 2026-02 ".parse()?;
# Ok(())
# }
```

Delimiters cannot be mixed within a single date — `"1991-08/15"` is an error.

---

## Programmatic construction

When you already have validated component values, use the `Year`, `Month`, and
`Day` newtypes:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::{FuzzyDate, Year, Month, Day};

let year  = Year::try_from(2026u16)?;
let month = Month::try_from(2u8)?;
let day   = Day::try_from(13u8)?;

let d = FuzzyDate::new_day(year, month, day)?;
let m = FuzzyDate::new_month(year, month)?;
let y = FuzzyDate::new_year(year)?;
# Ok(())
# }
```

A `(u16, Option<u8>, Option<u8>)` tuple converts via `TryFrom`, which is
equivalent to `from_columns`:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDate;

let date: FuzzyDate = (2026u16, Some(2u8), None).try_into()?;
// day without month is rejected
let err = FuzzyDate::try_from((2026u16, None, Some(13u8)));
assert!(err.is_err());
# Ok(())
# }
```

---

## Accessors

Components reflect the actual precision of the value. Nothing is invented.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDate;

let date: FuzzyDate = "2026-02".parse()?;

assert_eq!(date.year().get(), 2026u16);
assert_eq!(date.month().map(|m| m.get()), Some(2u8));
assert_eq!(date.day(), None);
# Ok(())
# }
```

`year()` always returns a `Year`. `month()` and `day()` return `Option<Month>`
and `Option<Day>` respectively.

---

## Bounds

Any `FuzzyDate` implies a concrete range of days it could represent. Bounds
give you the endpoints of that range.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDate;

let year:  FuzzyDate = "2026".parse()?;
let month: FuzzyDate = "2026-02".parse()?;
let day:   FuzzyDate = "2026-02-13".parse()?;

assert_eq!(year.lower_bound(),           (2026, 1, 1));
assert_eq!(year.upper_bound_inclusive(), (2026, 12, 31));

assert_eq!(month.lower_bound(),           (2026, 2, 1));
assert_eq!(month.upper_bound_inclusive(), (2026, 2, 28));
assert_eq!(month.upper_bound_exclusive(), Some((2026, 3, 1)));

// A day-precision date has equal lower and upper bounds
assert_eq!(day.lower_bound(),           (2026, 2, 13));
assert_eq!(day.upper_bound_inclusive(), (2026, 2, 13));
assert_eq!(day.upper_bound_exclusive(), Some((2026, 2, 14)));
# Ok(())
# }
```

`upper_bound_exclusive()` returns `None` for dates that cannot roll forward
(December 31, 9999).

---

## Ordering

Dates sort by lower bound. When two dates share the same lower bound,
less-precise sorts first.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDate;

let year:  FuzzyDate = "2026".parse()?;
let month: FuzzyDate = "2026-01".parse()?;
let day:   FuzzyDate = "2026-01-01".parse()?;

// All share lower bound 2026-01-01, but:
assert!(year < month);
assert!(month < day);

let mut dates = vec![day, year, month];
dates.sort();
assert_eq!(dates, vec![year, month, day]);
# Ok(())
# }
```

---

## Ranges

`FuzzyDateRange` pairs two `FuzzyDate` values. Each end can have independent
precision.

### Parsing

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDateRange;

let year_range:  FuzzyDateRange = "2020/2026".parse()?;
let month_range: FuzzyDateRange = "2020-01/2026-02".parse()?;
let mixed:       FuzzyDateRange = "2020-03/2026-02-13".parse()?;
# Ok(())
# }
```

### Construction

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::{FuzzyDate, FuzzyDateRange};

let start: FuzzyDate = "2020-03".parse()?;
let end:   FuzzyDate = "2026-02-13".parse()?;
let range = FuzzyDateRange::new(start, end)?;

// start > end is an error
let reversed = FuzzyDateRange::new(end, start);
assert!(reversed.is_err());
# Ok(())
# }
```

### Accessors

```rust
# #![allow(unused)]
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDateRange;

let range: FuzzyDateRange = "2020/2026".parse()?;

let start = range.start();
let end   = range.end();
let (start, end) = range.dates();
# Ok(())
# }
```

### Contains, overlaps, is_within

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::{FuzzyDate, FuzzyDateRange};

let range: FuzzyDateRange = "2020/2026".parse()?;

// Does a date fall within this range?
let inside:  FuzzyDate = "2023-06".parse()?;
let outside: FuzzyDate = "2019".parse()?;
assert!(range.contains(&inside));
assert!(!range.contains(&outside));

// Do two ranges share any time in common?
let overlapping: FuzzyDateRange = "2025/2030".parse()?;
let separate:    FuzzyDateRange = "2027/2030".parse()?;
assert!(range.overlaps(&overlapping));
assert!(!range.overlaps(&separate));

// Is one range entirely within another?
let inner: FuzzyDateRange = "2021/2024".parse()?;
assert!(inner.is_within(&range));
assert!(!range.is_within(&inner));
# Ok(())
# }
```

### Range bounds

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDateRange;

let range: FuzzyDateRange = "2020-03/2026-02-13".parse()?;

assert_eq!(range.lower_bound(),           (2020, 3, 1));
assert_eq!(range.upper_bound_inclusive(), (2026, 2, 13));
assert_eq!(range.upper_bound_exclusive(), Some((2026, 2, 14)));
# Ok(())
# }
```

### Range ordering

Ranges sort by start date, then by end date.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDateRange;

let a: FuzzyDateRange = "2020/2024".parse()?;
let b: FuzzyDateRange = "2022/2026".parse()?;
assert!(a < b);
# Ok(())
# }
```

---

## Database columns

`FuzzyDate` maps to three nullable columns: year (required), month (optional),
day (optional).

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDate;

let date: FuzzyDate = "2026-02".parse()?;

let (year, month, day) = date.to_columns();
assert_eq!((year, month, day), (2026u16, Some(2u8), None));

let restored = FuzzyDate::from_columns(2026, Some(2), None)?;
assert_eq!(date, restored);
# Ok(())
# }
```

`FuzzyDateRange` maps to six columns (three per end):

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDateRange;

let range: FuzzyDateRange = "2020-03/2026-02-13".parse()?;
let (sy, sm, sd, ey, em, ed) = range.to_columns();

let restored = FuzzyDateRange::from_columns(sy, sm, sd, ey, em, ed)?;
assert_eq!(range, restored);
# Ok(())
# }
```

---

## Serde

Both types serialize as ISO strings, preserving precision. Deserializing the
serialized form produces an equal value.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::{FuzzyDate, FuzzyDateRange};

let date: FuzzyDate = "2026-02".parse()?;
assert_eq!(serde_json::to_string(&date)?, r#""2026-02""#);

let restored: FuzzyDate = serde_json::from_str(r#""2026-02""#)?;
assert_eq!(date, restored);

let range: FuzzyDateRange = "2020-03/2026-02-13".parse()?;
assert_eq!(serde_json::to_string(&range)?, r#""2020-03/2026-02-13""#);
# Ok(())
# }
```

---

## Error handling

### ParseError

```rust
# #![allow(unused)]
use fuzzy_date::{FuzzyDate, ParseError};

match "2026-13".parse::<FuzzyDate>() {
    Err(ParseError::InvalidMonth(m))                     => { /* m = 13 */ }
    Err(ParseError::InvalidDay { month, day, year })     => { /* bad day for month */ }
    Err(ParseError::InvalidYear(y))                      => { /* y outside 1..=9999 */ }
    Err(ParseError::InvalidFormat(s))                    => { /* unrecognised format */ }
    Err(ParseError::EmptyInput)                          => { /* empty string */ }
    Ok(date)                                             => { /* valid */ }
}
```

### RangeError

```rust
# #![allow(unused)]
use fuzzy_date::{FuzzyDateRange, RangeError};

match "2026/2020".parse::<FuzzyDateRange>() {
    Err(RangeError::InvalidRange { start, end }) => { /* start > end */ }
    Err(RangeError::ParseError(e))               => { /* a date component failed */ }
    Err(RangeError::InvalidFormat(s))            => { /* wrong number of separators */ }
    Ok(range)                                    => { /* valid */ }
}
```
