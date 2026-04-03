AGENTS
======

Purpose
-------
This file documents how agentic coding assistants should operate inside this repository (camel-cup).
It includes build, lint, and test commands, conventions for code style, and operational rules for modifying the codebase.

Quick commands
--------------
- Build library: `cargo build`
- Run tests: `cargo test`
- Run a single integration test file: `cargo test --test move_camel`
- Run a single unit or doc test by name: `cargo test <test_name>` (use exact name or a substring)
- Run tests with verbose output: `cargo test -- --nocapture`
- Check formatting: `rustfmt --edition 2021 --check $(git ls-files "**/*.rs")` or simply `cargo fmt -- --check`
- Fix formatting: `cargo fmt`
- Static analysis / clippy: install via `rustup component add clippy` then run `cargo clippy -- -D warnings` (treat warnings as errors).

Project layout
--------------
- rust/: the Rust crate root with Cargo.toml and src/
  - src/state.rs: primary implementation (Camel enum and State)
  - src/lib.rs: exposes the public API
  - tests/: integration tests (move_camel.rs)

Testing notes
-------------
- Integration tests live in `rust/tests/` and exercise the public API via `use camel_cup::{Camel, State};`.
- To run a single integration test file: `cargo test --test <testname>` (example: `cargo test --test move_camel`).
- To run a single test case by name: `cargo test <test_fn_name>` (partial name matching allowed).
- Use `-- --nocapture` to see test stdout for debugging.

Coding Conventions
------------------
The repository uses idiomatic Rust. Follow these conventions when editing or adding code:

Formatting
- Use `rustfmt` for formatting. Run `cargo fmt` before committing. CI may enforce formatting with `cargo fmt -- --check`.
- Keep line length reasonable (80-100 chars). `rustfmt` settings are authoritative.

Imports
- Use explicit imports in module scope rather than glob imports (`use crate::module::Type` instead of `use crate::module::*`).
- Group imports: standard library first, external crates next, local modules last.

Types and Mutability
- Prefer immutable bindings (`let`) and only use `mut` when necessary.
- Use `BTreeMap` for maps where iteration in key order matters and `HashMap` otherwise.
- Prefer small, Copy-able enums for lightweight values (Camel is Copy).

Naming
- Use CamelCase for types and enums: `Camel`, `State`.
- Use UPPER_SNAKE_CASE for enum variants only when domain requires; otherwise use Rust's conventional `PascalCase` values. This project currently uses `WHITE`, `YELLOW`, etc. Keep existing names unchanged to avoid churn.
- Use snake_case for functions and variables: `move_camel`, `new_field`, `src_vec`, `position`.

Functions and APIs
- Prefer immutable, functional updates (return new State) unless mutation is required. Current `State::move_camel` returns a new State.
- Where an operation can fail due to invalid input, prefer non-panicking error handling (Result) for library-facing APIs. Internal scripts or tests may use panics only when invariants are violated. Current `move_camel` panics on invalid input; consider exposing a `try_move_camel` returning `Result` in future iterations.

Error Handling
- Use `Result<T, E>` with a small error enum for recoverable errors when building library functions that callers may use. For internal invariant violations or programming errors, `panic!` is acceptable but should include a descriptive message.
- When panicking in tests or small internal helpers, include the variable values in the message for easier debugging, e.g., `panic!("camel {:?} not found", camel)`.

Testing Practices
- Prefer explicit, deterministic tests. Integration tests should exercise the public API and assert on public types/fields.
- Keep test data explicit and avoid reproducing logic under test when asserting expected outcomes (i.e., write explicit expected results).
- Use helper functions inside test modules for setup/teardown only; avoid moving production logic into tests.

Git / Commits
- Commit messages should be short and descriptive. Use imperative mood: "add move_camel implementation", "add integration tests for move_camel".
- Don't commit build artifacts in rust/target/ (these are currently present locally but should be ignored by .gitignore). If needed, add or update .gitignore to exclude `rust/target/`.

Agent Operational Rules
-----------------------
- Always run `cargo test` and `cargo fmt` locally before proposing Pull Requests.
- When adding new public APIs, add unit tests and integration tests demonstrating expected behavior.
- Make minimal, incremental changes. Prefer small, reviewable commits.
- If you encounter ambiguous requirements, ask a concise clarifying question rather than guessing. Provide 1-2 implementation options when appropriate.

Cursor / Copilot rules
----------------------
- If this repository contains `.cursor` or Copilot-specific instruction files, follow them. (No such files were found.)
- If project maintains `.github/copilot-instructions.md`, follow those instructions in addition to this file.

Local developer setup
---------------------
- Install Rust via rustup: https://rustup.rs
- Install clippy and fmt: `rustup component add clippy rustfmt`
- Run `cargo build` to compile and `cargo test` to verify.

Examples
--------
- Run all tests: `cargo test`
- Run a single integration test file: `cargo test --test move_camel`
- Run a single test case: `cargo test test_move_white_combinations`
- Check formatting: `cargo fmt -- --check`

Contacts and escalation
-----------------------
- If tests fail after your change, reproduce locally, run `cargo test -- --nocapture`, and include the failing test output in the issue or PR description.
- For non-obvious behavior involving game rules, ask the repository owner for clarification.

Maintenance notes
-----------------
- Keep AGENTS.md up to date when adding new tools, CI, or style rules.
- If a linter or formatting tool is added to CI, include exact commands and required versions here.

End
