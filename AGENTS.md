AGENTS
======

Purpose
-------
Short, high-signal instructions for automated agents working in this repository.
Only include facts an agent would likely miss (non-obvious commands, API renames,
repo vs crate name mismatches, and safety steps for rewriting history).

Quick Commands (exact)
----------------------
# work in the Rust crate directory
cd rust

# build / quick verify
cargo check
cargo build

# run everything
cargo test

# run a single integration test file (file name without .rs):
cargo test --test desert_tiles

# run a single test case (substring or exact):
cargo test test_move_white_combinations

# formatting and lint
rustup component add rustfmt clippy
cargo fmt --all
cargo fmt -- --check
cargo clippy -- -D warnings

Repository layout & naming gotchas
---------------------------------
- Repo name: `camel-up` (filesystem root). Crate name inside Rust manifest is `camel_up` (see rust/Cargo.toml). After this change imports should use `use camel_up::{...}`.
- Primary crate: rust/ (Cargo.toml, src/, tests/). Treat `rust/` as the workspace package.
- Public API is exported from rust/src/lib.rs: Camel, DesertTile, Space, State.

High-value API facts (verify before changing code)
-------------------------------------------------
- Camel enum: variants are PascalCase: `Camel::White`, `Camel::Yellow`, `Camel::Orange`, `Camel::Green`, `Camel::Blue`.
- Field name changed to Space: enum `Space::Camels(Vec<Camel>)` and `Space::Desert(DesertTile)`.
- API renames to keep in mind:
  - `State::move_unit(&self, camel: Camel, steps: u8) -> (State, Option<u8>)`
  - `State::move_multiple_units` (accepts IntoIterator of (Camel,u8))
  - `State::simulate_outcomes()` and `State::evaluate_desert_placements()`

Important invariants and behaviour
----------------------------------
- Keys and steps use `u8` (valid keys 1..=16). Code uses checked_add/checked_sub; watch for over/underflow and preserve u8 semantics.
- Desert tiles: Oasis forwards the moving unit, Mirage moves it back. Precondition: deserts must not be adjacent in the direction of the effect.
- Many functions panic on invalid input (e.g. move_unit when steps==0 or camel not found). Tests rely on these panics in places.

Testing and debug tips
----------------------
- Integration tests live under rust/tests/*.rs. Use `cargo test --test <name>` to run one file.
- To reproduce failing test output use `cargo test <test_name> -- --nocapture`.
- `cargo check` is usually fast and sufficient before running full tests.

Editing, commits and history rules
--------------------------------
- Don't commit build artifacts (rust/target/). .gitignore is present but check before committing.
- When rebasing or force-pushing: create a safety backup branch first, e.g.
  git -C /path/to/repo branch backup/<branch>-before-rebase <branch>
- Use `git push --force-with-lease` when updating branches on origin to avoid clobbering others' work.

Operational gotchas an agent would miss
--------------------------------------
- Repo != crate name: before this change many references used `use camel_cup::{...}`. Update imports to `use camel_up::{...}` after renaming Cargo.toml.
- Tests and code expect u8 arithmetic semantics; changing types to i32 or removing checked arithmetic will break invariants/tests.
- There used to be a worktree for branches; check `git worktree list` if branch deletion fails.

If something is unclear
-----------------------
- Prefer reading rust/Cargo.toml and rust/src/lib.rs for the true public API. Trust code over README when they disagree.
- If a change affects public API names (crate exports), run `cargo check` and the full `cargo test` suite before pushing.

Maintenance
-----------
- Keep this file minimal and factual. Add only repository-specific rules that an automated agent would not infer.
