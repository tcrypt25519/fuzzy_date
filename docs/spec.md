# fuzzy_date

A tiny Rust crate to represent dates with varying precision (year, month-year, full),
without inventing missing parts. It parses common ISO (`YYYY`, `YYYY-MM`, `YYYY-MM-DD`)
and US month-first (`MM/YYYY`, `MM/DD/YYYY`) formats, formats in ISO, and exposes range
helpers for lower/upper bounds.

## Highlights

- Enum representation: `FuzzyDate::Year`, `FuzzyDate::Month`, `FuzzyDate::Day`
- ISO `Display`: `YYYY`, `YYYY-MM`, `YYYY-MM-DD`
- Parsing: ISO and common US month-first inputs; trims inner whitespace
- No fake data: compare using earliest possible date, then precision
- Range helpers: `lower_bound()`, `upper_bound_inclusive()`, `upper_bound_exclusive()`
- `FuzzyDateRange` for ranges; parsed/formatted as `start/end`
- `serde` support: serializes as ISO strings

## Quick start

- `cargo test` to run the included tests.
- Use `FuzzyDate::from_str("1991-08")?` or `"08/1991"`, etc.
