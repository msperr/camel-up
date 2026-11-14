camel-up
=========

A small Rust library implementing a minimal model for a camel race game:
 - Camel enum (White, Yellow, Orange, Green, Blue)
 - State struct with a BTreeMap-based space layout (Space) and move logic
   - Public API highlights: State::move_unit, State::move_multiple_units, State::simulate_outcomes, State::evaluate_desert_placements
   - `State::simulate_outcomes(&already_moved)` simulates only the camels that have not moved yet; pass an empty slice for a full leg. Passing all five camels panics because the leg is already over. In partial mode the returned desert counts cover only the remaining moves; hits from already-revealed moves are not included.
 - Combinatorics utilities: Permutations and Product iterators
 - Integration tests exercising combinatorics and state move semantics

Purpose
-------
This project helps a player playing Camel Up evaluate which action is the best
and promises the highest number of coins. It models the coin-relevant mechanics
so that candidate actions can be compared by their expected Egyptian Pound (EP)
payout.

The rules used for that evaluation are documented in [RULES.md](RULES.md). The
current implementation models the 2014 base game; the rules document also
summarises the 2018 and 2024 edition differences.

Game rules
----------
- Official Camel Up (2014) rulebook, English (PDF):
  https://cdn.1j1ju.com/medias/33/c8/d5-camel-up-rulebook.pdf
- Cross-check (transcription):
  https://www.ultraboardgames.com/camel-up/game-rules.php
- Publisher page (current edition):
  https://www.lookout-spiele.de/en/games/camelup.html
- Full coin-relevant rules used by this repository: [RULES.md](RULES.md)

> The rules document covers the full base game; not every rule is implemented in
> the crate yet. The current Rust model covers camel movement/stacking and
> desert tiles.

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
- RULES.md - coin-relevant Camel Up rules used by the project

Contributing
------------
- Run `cargo fmt` and `cargo clippy -- -D warnings` before creating a PR.
- Add tests for new public API surface.
- Keep commits small and focused; use imperative commit messages.

Contact / License
-----------------
- Repository: https://github.com/msperr/camel-up
- Add a LICENSE file (MIT/Apache2 suggested) if you want a clear open-source license.

