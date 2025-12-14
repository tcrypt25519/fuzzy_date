# fuzzy_date

A Rust crate to represent dates with varying precision (year, month-year, full day),
without inventing missing parts. It parses common ISO (`YYYY`, `YYYY-MM`, `YYYY-MM-DD`)
and US month-first (`MM/YYYY`, `MM/DD/YYYY`) formats, formats output in ISO, and
exposes range helpers for lower/upper bounds.

## Features

- **Enum representation**: `FuzzyDate::Year`, `FuzzyDate::Month`, `FuzzyDate::Day`
- **ISO Display**: Outputs as `YYYY`, `YYYY-MM`, or `YYYY-MM-DD`
- **Flexible parsing**: Accepts ISO and US month-first inputs with whitespace trimming
- **No fake data**: Comparisons use the earliest possible date, then precision
- **Bound helpers**: `lower_bound()`, `upper_bound_inclusive()`, and `upper_bound_exclusive()`
- **Date ranges**: `FuzzyDateRange` represents a range between two fuzzy dates
- **Serde support**: Serializes as ISO strings

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
fuzzy_date = { path = "crates/fuzzy_date" }
```

### Parsing dates

```rust
use std::str::FromStr;
use fuzzy_date::FuzzyDate;

// ISO format
let year_only = FuzzyDate::from_str("1991")?;
let month_year = FuzzyDate::from_str("1991-08")?;
let full_date = FuzzyDate::from_str("1991-08-15")?;

// US month-first format
let month_year_us = FuzzyDate::from_str("08/1991")?;
let full_date_us = FuzzyDate::from_str("08/15/1991")?;
```

### Accessing components

```rust
use fuzzy_date::FuzzyDate;

let date: FuzzyDate = "1991-08-15".parse()?;
assert_eq!(date.year().get(), 1991);
assert_eq!(date.month().map(|m| m.get()), Some(8));
assert_eq!(date.day().map(|d| d.get()), Some(15));
```

### Date bounds

```rust
use fuzzy_date::FuzzyDate;

let month: FuzzyDate = "1991-08".parse()?;

// Earliest possible concrete date
assert_eq!(month.lower_bound(), (1991, 8, 1));

// Latest possible concrete date (inclusive)
assert_eq!(month.upper_bound_inclusive(), (1991, 8, 31));

// Exclusive upper bound for range queries
assert_eq!(month.upper_bound_exclusive(), Some((1991, 9, 1)));
```

### Date ranges

`FuzzyDateRange` represents a range between two fuzzy dates. Ranges are parsed
using `/` as the separator and formatted in the same way.

#### Range parsing

```rust
use fuzzy_date::FuzzyDateRange;

// Year precision
let range: FuzzyDateRange = "1990/2000".parse()?;

// Month precision
let range: FuzzyDateRange = "1990-01/2000-12".parse()?;

// Day precision
let range: FuzzyDateRange = "1990-01-15/2000-12-31".parse()?;

// Mixed precision (start and end can have different precision)
let range: FuzzyDateRange = "1991-08/2025".parse()?;
let range: FuzzyDateRange = "1990/2025-12-31".parse()?;
```

#### Range formatting

Ranges are formatted as `{start}/{end}` using ISO format for each date:

```rust
use fuzzy_date::FuzzyDateRange;

let range: FuzzyDateRange = "1991-08/2025".parse()?;
assert_eq!(range.to_string(), "1991-08/2025");

let range: FuzzyDateRange = "1990-01-15/2000-12-31".parse()?;
assert_eq!(range.to_string(), "1990-01-15/2000-12-31");
```

#### Range operations

```rust
use fuzzy_date::FuzzyDateRange;

let range: FuzzyDateRange = "1990/2000".parse()?;

// Access start and end
let (start, end) = range.dates();

// Check containment
let date = "1995".parse()?;
assert!(range.contains(&date));

// Check overlap with another range
let other: FuzzyDateRange = "1998/2005".parse()?;
assert!(range.overlaps(&other));

// Check if one range is within another
let inner: FuzzyDateRange = "1992/1998".parse()?;
assert!(inner.is_within(&range));
```

### Ordering

Dates are ordered by their earliest possible concrete date. When two dates have
the same lower bound, less precise dates come first:

```rust
use fuzzy_date::FuzzyDate;

let year: FuzzyDate = "1991".parse()?;
let month: FuzzyDate = "1991-01".parse()?;
let day: FuzzyDate = "1991-01-01".parse()?;

// All have the same lower bound (1991-01-01), but:
assert!(year < month);  // Year is less precise
assert!(month < day);   // Month is less precise than day
```

### Database columns

Convert to and from database column representation:

```rust
use fuzzy_date::FuzzyDate;

let date: FuzzyDate = "1991-08".parse()?;

// To columns: (year, Option<month>, Option<day>)
let (year, month, day) = date.to_columns();
assert_eq!((year, month, day), (1991, Some(8), None));

// From columns
let restored = FuzzyDate::from_columns(1991, Some(8), None)?;
assert_eq!(date, restored);
```

## Supported formats

| Format       | Example      | Precision |
| ------------ | ------------ | --------- |
| `YYYY`       | `1991`       | Year      |
| `YYYY-MM`    | `1991-08`    | Month     |
| `YYYY-MM-DD` | `1991-08-15` | Day       |
| `MM/YYYY`    | `08/1991`    | Month     |
| `MM/DD/YYYY` | `08/15/1991` | Day       |

Whitespace around components is trimmed, but delimiters cannot be mixed
(use either `-` for ISO or `/` for US format within a single date).

## Running tests

```sh
cargo test -p fuzzy_date
```
