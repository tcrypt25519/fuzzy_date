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

## Error Behavior

`FromStr` returns `ParseError`, an enum with these variants:

| Variant                          | When raised                                              |
|----------------------------------|----------------------------------------------------------|
| `EmptyInput`                     | Input is empty or all whitespace                         |
| `InvalidFormat(String)`          | Unrecognized structure, mixed delimiters, non-digit tokens |
| `InvalidYear(u16)`               | Year is 0 or exceeds 9999                               |
| `InvalidMonth(u8)`               | Month is 0 or exceeds 12                                |
| `InvalidDay { month, day, year }`| Day does not exist in the given month/year              |

All errors are diagnostic — each carries enough context to identify the offending
value. Parsing is pure computation: no I/O, no allocation beyond the error message.

## Module Architecture

The crate is organized into four private modules with a single public re-export layer:

| Module    | Role                                                                              |
|-----------|-----------------------------------------------------------------------------------|
| `lib`     | Public API: `FuzzyDate`, `ParseError`, `FromStr`, `Ord`, serde impls             |
| `types`   | Validated newtypes: `Year`, `Month`, `Day` (backed by `NonZeroU16`/`NonZeroU8`)  |
| `consts`  | Named constants: month lengths, `MAX_YEAR`, `DAYS_IN_MONTH`, separator chars     |
| `range`   | `FuzzyDateRange` and `RangeError`                                                 |
| `prelude` | Internal re-exports shared across modules                                         |

Dependency direction is strictly one-way: `lib` and `range` import from `types` and
`consts`; no leaf module imports from `lib` or `range`. This keeps compilation
incremental and prevents circular dependencies.
