# Contributing

Thanks for your interest in contributing to `fuzzy_date`.

## Style

- Format with `cargo fmt` before committing (`make fmt`).
- Fix all clippy warnings — the project runs clippy at `deny` level (`make clippy`).
- Add or update rustdoc comments for any public items you add or change.
- Keep changes focused; one logical change per commit/PR.

## Tests

- Every bug fix should include a regression test.
- Every new parsing rule or API surface should have unit tests.
- Run the full suite before opening a PR: `make ci` (format-check + clippy + tests + docs).

## Opening a PR

1. Fork the repository and create a feature branch from `main`.
2. Run `make ci` locally and confirm it passes.
3. Open a PR against `main` with a short description of what and why.

## Expectations

- Keep it small: PRs that are easier to review get merged faster.
- The crate has no runtime dependencies beyond `serde` and `thiserror`; please
  avoid introducing new ones without discussion.
- Minimum supported Rust version is tracked in `Cargo.toml` (`rust-version`);
  do not use features newer than that without bumping it.
