camel-up
=========

A small Rust library implementing a minimal model for a "camel cup" game:
- Camel enum (WHITE, YELLOW, ORANGE, GREEN, BLUE)
- State struct with a BTreeMap-based field layout and move logic
- Combinatorics utilities: Permutations and Product iterators
- Integration tests exercising combinatorics and state move semantics

Quickstart / Prerequisites
--------------------------
- Rust toolchain (rustup): https://rustup.rs
- Install components:
  - rustfmt and clippy: `rustup component add rustfmt clippy`
- Ensure `gh` (GitHub CLI) is available for repo operations (optional)

Build / Run / Test
------------------
- Build: `cargo build`
- Run all tests: `cargo test`
- Run a single integration test file:
  - `cargo test --test combinatorics`
  - `cargo test --test state`
- Run a single test by name (substring match): `cargo test <test_name>`
- Run tests verbosely / show output: `cargo test -- --nocapture`

Formatting / Linting
--------------------
- Format code: `cargo fmt --all`
- Check formatting: `cargo fmt -- --check`
- Lint with clippy (treat warnings as errors): `cargo clippy -- -D warnings`

Repository Layout
-----------------
- rust/
  - Cargo.toml
  - src/
    - lib.rs
    - state.rs
    - combinatorics.rs
  - tests/
    - combinatorics.rs
    - state.rs
- AGENTS.md - contributor & agent guidance

Contributing
------------
- Run `cargo fmt` and `cargo clippy -- -D warnings` before creating a PR.
- Add tests for new public API surface.
- Keep commits small and focused; use imperative commit messages.

Contact / License
-----------------
- Repository: https://github.com/msperr/camel-up
- Add a LICENSE file (MIT/Apache2 suggested) if you want a clear open-source license.


Recent changes
---------------
- Repository renamed from camel-cup to camel-up.
- Consolidated agent guidance and documentation updates.
