# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-02-13

### Added

- Initial standalone release, extracted from the Atlas workspace
- `FuzzyDate` enum for representing dates with year, month, or day precision
- `FuzzyDateRange` for representing a range between two fuzzy dates
- ISO and US month-first format parsing
- `lower_bound()`, `upper_bound_inclusive()`, `upper_bound_exclusive()` helpers
- `to_columns()` / `from_columns()` for database representation
- `contains()`, `overlaps()`, `is_within()` range operations
- Serde support (serializes/deserializes as ISO strings)
- Strict delimiter validation (no mixed `-` and `/` within a single date)
