camel-up
=========

A small Rust library implementing a minimal model for a camel race game:
 - Camel enum (White, Yellow, Orange, Green, Blue)
 - State struct with a BTreeMap-based space layout (Space) and move logic
   - Public API highlights: State::move_unit, State::move_multiple_units, State::simulate_outcomes, State::evaluate_desert_placements
   - `State::simulate_outcomes(&already_moved)` simulates only the camels that have not moved yet; pass an empty slice for a full leg. Passing all five camels panics because the leg is already over. In partial mode the returned desert counts cover only the remaining moves; hits from already-revealed moves are not included.
 - Expected leg-bet payouts: `expected_leg_bet_coins(state, available, already_moved)` and the `AvailableLegBets` tile-stack type, derived from the exhaustive outcome enumeration
 - Combinatorics utilities: Permutations and Product iterators
 - Integration tests exercising combinatorics and state move semantics

Purpose
-------
This project helps a Camel Up player evaluate which action has the highest
expected Egyptian Pound (EP) payout. It models the coin-relevant mechanics so
that candidate actions can be compared by expected EP.

This repository covers the rules of the **2014 game**. The 2018 and later
editions are out of scope; the rules used here are documented in
[RULES.md](RULES.md).

Game rules
----------
- Official Camel Up (2014) rulebook, English (PDF):
  https://cdn.1j1ju.com/medias/33/c8/d5-camel-up-rulebook.pdf
- Cross-check (transcription):
  https://www.ultraboardgames.com/camel-up/game-rules.php
- Full coin-relevant rules used by this repository: [RULES.md](RULES.md)

> The rules document covers the coin-relevant parts of the 2014 base game; not
> every rule is implemented in the crate yet. The current Rust model covers
> camel movement/stacking and desert tiles.

Quickstart / Prerequisites
--------------------------
- Rust toolchain (rustup): https://rustup.rs
- Install components:
  - rustfmt and clippy: `rustup component add rustfmt clippy`
- Frontend: Node.js 22.12 or newer and npm
- Ensure `gh` (GitHub CLI) is available for repo operations (optional)

Build / Run / Test
------------------
- Build: `cargo build`
- Run all tests: `cargo test`
- Run a single integration test file:
  - `cargo test --test combinatorics`
  - `cargo test --test desert_tiles`
  - `cargo test --test leg_bets`
  - `cargo test --test state`
- Run a single test by name (substring match): `cargo test <test_name>`
- Run tests verbosely / show output: `cargo test -- --nocapture`

Formatting / Linting
--------------------
- Format code: `cargo fmt --all`
- Check formatting: `cargo fmt -- --check`
- Lint with clippy (treat warnings as errors): `cargo clippy -- -D warnings`
- Frontend: `cd frontend && npm run lint && npm run format:check`

Frontend
--------
The frontend is a single-page TypeScript app (React + Vite) that renders the
centred square SVG board: the 16 dark Camel Up fields run clockwise around the
light central panel, with small numbers on the inner edges and a centred 3 x 1
tray with five dice slots. The tray's plus button opens an add-die dialog; each
thrown die moves a coloured camel meeple onto the field, with camel-unit
stacking and a finish-line overflow area. Clear removes dice and camels, while
Next round clears only the dice once all five are on the field. The read-only
cockpit below the board gets its probabilities and expected coins from the Rust
backend; camel positions remain local UI state. See
[frontend/README.md](frontend/README.md).

```sh
# terminal 1: backend API on http://127.0.0.1:3000
cd rust
cargo run

# terminal 2: frontend on http://127.0.0.1:5173 (proxies /api to the backend)
cd frontend
npm install
npm run dev -- --host 127.0.0.1
npm run build     # type-check and production build
npm test          # unit and component tests
```

The frontend follows the repository's frontend architecture decisions
(D-01 to D-07); see [frontend/README.md](frontend/README.md).

Repository Layout
-----------------
- rust/
  - Cargo.toml
  - src/
    - lib.rs
    - state.rs
    - leg_bets.rs
    - combinatorics.rs
  - tests/
    - combinatorics.rs
    - desert_tiles.rs
    - leg_bets.rs
    - state.rs
- frontend/
  - index.html
  - src/
    - App.tsx - the single page, SVG board, dice, camels, and add-die dialog
    - boardLayout.ts - static space-to-coordinate and camel-stack geometry
    - camelRace.ts - local camel positions and die-driven movement
    - main.tsx
    - styles.css
  - README.md - frontend setup and scope
- AGENTS.md - contributor & agent guidance
- RULES.md - coin-relevant Camel Up rules used by the project
- ROADMAP.md - outstanding rules-verification findings

Contributing
------------
- Run `cargo fmt` and `cargo clippy -- -D warnings` before creating a PR.
- Add tests for new public API surface.
- Keep commits small and focused; use imperative commit messages.

Contact / License
-----------------
- Repository: https://github.com/msperr/camel-up
- Add a LICENSE file (MIT/Apache2 suggested) if you want a clear open-source license.

