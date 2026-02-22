# fuzzy_date

A Rust crate for dates with varying precision: year-only, year+month, or full date.

## Background

Real-world date data often isn't complete. Vendors, forms, and manual records
frequently give you a month and year but no day, or just a year. The typical
response is to invent the missing parts — pick the first of the month, or the
last — but that introduces data you never actually had. Once fabricated
precision is stored alongside real precision, the two are indistinguishable.

I built this because I kept running into it in data pipelines and wanted a type
that carries exactly the precision the source provides, nothing more.

## Model

`FuzzyDate` is an enum with three variants — year, month, and day precision.
`FuzzyDateRange` pairs two `FuzzyDate` values, each end independent.

Because dates of different precision can't be meaningfully compared for equality,
the API works with bounds and overlap instead:

```rust
# use fuzzy_date::FuzzyDate;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let month: FuzzyDate = "2026-02".parse()?;

assert_eq!(month.lower_bound(), (2026, 2, 1));
assert_eq!(month.upper_bound_inclusive(), (2026, 2, 28));

let day: FuzzyDate = "2026-02-13".parse()?;
assert!(month.contains(&day));  // Feb 13 falls within February
# Ok(())
# }
```

Ordering sorts by lower bound; when two dates share the same lower bound,
less-precise sorts first (`2026` < `2026-02` < `2026-02-01`).

## Usage

```toml
[dependencies]
fuzzy_date = "0.1"
```

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use fuzzy_date::FuzzyDate;

let a: FuzzyDate = "2026-02-13".parse()?;  // day precision
let b: FuzzyDate = "2026-02".parse()?;     // month precision
let c: FuzzyDate = "2026".parse()?;        // year precision

// Components reflect actual precision — nothing is invented
assert_eq!(b.year().get(), 2026);
assert_eq!(b.month().map(|m| m.get()), Some(2));
assert_eq!(b.day(), None);
# Ok(())
# }
```

Serialize/deserialize as ISO strings preserving precision. For database
storage, `to_columns()` / `from_columns()` map to three nullable columns
(year required, month and day optional). Ranges use six.

## Supported formats

| Format       | Example       | Precision |
|--------------|---------------|-----------|
| `YYYY`       | `2026`        | Year      |
| `YYYY-MM`    | `2026-02`     | Month     |
| `YYYY-MM-DD` | `2026-02-13`  | Day       |
| `MM/YYYY`    | `02/2026`     | Month     |
| `MM/DD/YYYY` | `02/13/2026`  | Day       |

Ranges parse as `{start}/{end}` — e.g. `2020-03/2026-02-13`.

## Rejected formats

| Format           | Example          | Reason                                          |
|------------------|------------------|-------------------------------------------------|
| `MM-DD-YYYY`     | `02-13-2026`     | Hyphens require year-first; ambiguous with ISO  |
| `DD/MM/YYYY`     | `13/02/2026`     | Day-first slash not supported                   |
| `YYYYMMDD`       | `20260213`       | No separator; parsed as bare year or error      |
| Mixed delimiters | `2026-02/13`     | Any mix of `-` and `/` is rejected immediately  |
| With time        | `2026-02-13T10:00` | Time components are not accepted              |

For fuller examples including ranges, database integration, construction, and
error handling, see [docs/examples.md](docs/examples.md).

## Security and Validation

Parsing is pure computation — no `unsafe` code, no I/O, no external calls.

Every component is validated before a `FuzzyDate` is constructed:

- Year must be 1–9999.
- Month must be 1–12.
- Day must be valid for the given month and year (leap years handled correctly).

A `FuzzyDate` value is always well-formed; the type system makes invalid dates
unrepresentable through the public API.
