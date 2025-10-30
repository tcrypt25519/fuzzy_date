# partial_date

A tiny Rust crate to represent dates with varying precision (year, month-year, full),
without inventing missing parts. It parses common ISO (`YYYY`, `YYYY-MM`, `YYYY-MM-DD`)
and US month-first (`MM/YYYY`, `MM/DD/YYYY`) formats, formats in ISO, and exposes range
helpers for lower/upper bounds.

## Highlights
- Enum representation: `Year`, `MonthYear`, `Full`
- ISO `Display`: `YYYY`, `YYYY-MM`, `YYYY-MM-DD`
- Parsing: ISO and common US month-first inputs; trims inner whitespace
- No fake data: compare using earliest possible date, then precision
- Range helpers: `lower_bound()` and `upper_bound_exclusive()`
- `serde` support with `snake_case` tag; legacy alias `monthyear`

## Quick start
- `cargo test` to run the included tests.
- Use `PartialDate::from_str("1991-08")?` or `"08/1991"`, etc.
