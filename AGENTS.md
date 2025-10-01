# Repository Guidelines

## Project Structure & Module Organization
The workspace is defined in Cargo.toml with crates under crates/ (core runtime, persistence, collaboration, macros). Shared CLI utilities live in 	ools/. Integration demos are in examples/, with the Tauri front end in examples/demo2/src-tauri. Bench and performance suites sit in enches/ and enchmarks/. Test fixtures and large JSON templates reside in 	est-data/. Design specs and functional docs are kept under design-docs/.

## Build, Test, and Development Commands
- cargo check ? fast type-check the entire workspace before staging changes.
- cargo fmt --all ? apply rustfmt (see ustfmt.toml) to keep style consistent.
- cargo clippy --workspace --all-targets -- -D warnings ? run lint gates that mirror CI.
- cargo test --workspace ? execute unit and integration tests; use --package moduforge-core for focused runs.
- cargo bench ? launch Criterion benches in enches/ when performance regressions are suspected.

## Coding Style & Naming Conventions
Rust 2024 edition with 4-space indentation and an 80-character soft wrap (see ustfmt.toml). Prefer snake_case for modules and functions, UpperCamelCase for types, and SCREAMING_SNAKE_CASE for constants. Group related derives on a single line, and keep use ordering intentional because eorder_imports is disabled. Public APIs should document panics and error cases; mirror existing module doc comments.

## Testing Guidelines
Unit tests sit beside sources in each crate; integration suites use the 	ests/ folders. Name test files after the module under test, e.g. state/tests/persistence.rs. Reproduce collaboration flows with fixtures from 	est-data/. Target high-value code paths first; aim for meaningful assertions rather than numeric coverage. Add Criterion scenarios whenever algorithmic complexity changes.

## Commit & Pull Request Guidelines
Commit history favors concise, imperative summaries (often in Chinese), e.g. ?? deno ?????. Keep related changes squashed; include scope or crate name when helpful. Pull requests should describe motivation, link tracking issues, and note breaking changes. Attach screenshots or GIFs for front-end tweaks in examples/demo2. Ensure the CI commands above pass locally before requesting review.

## Security & Configuration Tips
Never commit secrets; use .env.local patterns ignored by Git. Review scripts/ outputs before running with elevated permissions. When introducing new services, document ports and auth requirements in design-docs/ for future agents.
