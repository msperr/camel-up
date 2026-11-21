# ROADMAP

Outstanding findings from the one-time verification of the existing Rust
implementation against the rules documented in [RULES.md](RULES.md).

The verified target is the **2014 base game** (5 racing camels, 5 racing dice,
3 leg-betting tiles per camel, desert tiles). The code was not changed during
that verification; this file records what still needs to be done.

Classification
--------------
- **Bug** — current code contradicts the documented rules or produces an
  incorrect result.
- **Feature** — a rule or mechanic is not implemented at all.
- **Wrong behaviour** — for a legal game state, the code does the wrong thing.
- **Missing feature** — no implementation exists yet. The subset that does exist
  may still be correct for what it claims to do.
- **Latent** — only reachable through malformed/internal state, not normal play.
- **Cosmetic** — no effect on rules, coins, or results.

At a glance
-----------
| Type | Wrong behaviour | Latent | Missing feature | Cosmetic | Total |
|---|---:|---:|---:|---:|---:|
| Bug | 1 | 0 | 0 | 0 | 1 |
| Feature | 0 | 0 | 8 | 0 | 8 |

Resolved since the verification: **B-01, B-02, B-04, B-05**. B-03 is retained
by design: a desert behind the rearmost camel can never be reached (camel
positions never decrease across a move), so excluding those spaces loses no
coin-relevant outcomes.

Priority: **P0** blocks the repository purpose (evaluating actions by expected
coins); **P1** is required for a usable evaluator; **P2** is completeness or
robustness; **P3** is cleanup.

Bugs
----

### B-01 — Finish line is not enforced — P0

- **Type:** Bug
- **Classification:** Wrong behaviour
- **Where:** `State::move_unit`, `MAX_FIELD` in `rust/src/state.rs`
- **Finding:** A camel on space 16 can be moved to space 17 or beyond. The race
  does not end, and no final leg scoring is performed. `MAX_FIELD` is only used
  by `evaluate_desert_placements`, so movement and evaluation disagree about the
  board boundary.
- **Rules:** The race ends immediately when a camel unit crosses the finish line
  (past space 16); the current leg is then scored one last time, followed by the
  overall bets.
- **Impact:** Endgame states and expected payouts are wrong or impossible to
  compute.
- **Next step:** Detect the crossing, stop the movement, and expose a
  race-finished state; use the same boundary in movement and evaluation; add
  tests for exact finish and overshoot.
- **Status:** Resolved on `fix/rust-core-bugs`. `MAX_FIELD` is now shared with
  the Monte Carlo module, `move_unit` clamps a crossing unit onto space 16 and
  sets `State::is_finished()`, and `move_multiple_units` stops at the finish.

### B-02 — Ranking index is reversed in simulation — P0

- **Type:** Bug
- **Classification:** Wrong behaviour (contract ambiguity)
- **Where:** `State::order`, `State::simulate_outcomes`
- **Finding:** `order()` returns camels from least advanced to most advanced
  (rear-to-front, bottom-to-top). `simulate_outcomes` uses that order directly
  for its per-position counts, so index 0 is the least advanced camel, not 1st
  place. If consumers read index 0 as "winner", the probability distribution is
  inverted.
- **Rules:** Ranking is: top of the most advanced stack, then each camel below it
  in that stack, then the top of the next stack, and so on.
- **Impact:** Leg probabilities and any EV derived from them can be reversed.
- **Next step:** Add an explicit ranking API that returns 1st through last
  (reverse of `order()`), use it in `simulate_outcomes`, document the index
  convention, and test multi-camel stacks where 1st and 2nd are in the same
  stack.
- **Status:** Resolved on `fix/rust-core-bugs`. `State::ranking()` returns 1st
  through last, `simulate_outcomes` uses it (index 0 is 1st), and the convention
  is documented and tested.

### B-03 — Desert candidate list misses legal spaces — P2

- **Type:** Bug
- **Classification:** Wrong behaviour (incomplete enumeration)
- **Where:** `State::evaluate_desert_placements`
- **Finding:** The loop starts at `min_key` (the lowest occupied space), so it
  never considers empty spaces below it. Those spaces can be legal desert
  placements.
- **Rules:** A desert tile may be placed on any empty space except space 1,
  provided it is not adjacent to another desert tile.
- **Impact:** The API does not actually evaluate "each position"; in the 2014
  game a tile behind all camels is likely to be inert, so the immediate coin
  impact is low, but the candidate set is still incomplete.
- **Next step:** Iterate the full legal track range (for example 2..=16)
  independently of `min_key`; add a test with a state whose rearmost camel is
  ahead of unused spaces.
- **Status:** Retained by design. A camel's position never decreases across a
  move (dice and oasis move forward; a mirage moves back one from a space the
  unit just reached by moving forward), so a desert strictly behind the
  rearmost camel cannot be triggered. No coin-relevant outcomes are lost.

### B-04 — Space 1 is not explicitly excluded — P3

- **Type:** Bug
- **Classification:** Latent
- **Where:** `State::evaluate_desert_placements`
- **Finding:** There is no explicit "not space 1" check. The loop currently
  starts at `min_key`, so in normal states it never proposes space 1; a
  malformed state with a key below 1 (or a future fix that iterates from 1)
  could. Normal setup can leave space 1 empty, so occupancy alone is not a
  sufficient guard.
- **Rules:** A desert tile may never be placed on space 1.
- **Impact:** Only malformed/internal states are affected.
- **Next step:** Add the explicit `pos != 1` check and a regression test. The
  proposed B-03 range (`2..=16`) would also avoid this.
- **Status:** Resolved on `fix/rust-core-bugs` with an explicit `pos == 1`
  guard and a regression test using a malformed key below 1.

### B-05 — Stale panic message — P3

- **Type:** Bug
- **Classification:** Cosmetic (not wrong rules behaviour, not a missing feature)
- **Where:** `State::simulate_outcomes`
- **Finding:** The panic message refers to `tally_outcomes`, an old API name.
- **Impact:** Diagnostics only.
- **Next step:** Rename the message to `simulate_outcomes`.
- **Status:** Already resolved in the code at verification time (the roadmap
  entry was stale): all panic messages in `simulate_outcomes` use the current
  name, and no `tally_outcomes` text remains outside this document.

Features
--------

### F-01 — Player, money, turn, and leg state — P0

- **Type:** Feature
- **Classification:** Missing feature
- **Finding:** The crate has no model for player money, the starting-player
  marker, whose turn it is, remaining dice, pyramid tickets held, leg-betting
  tiles held, overall-bet card decks, or desert-tile ownership.
- **Impact:** The repository cannot yet choose among the four actions or compare
  their expected Egyptian Pound (EP) outcomes.
- **Next step:** Add a game state that composes the existing `State` with the
  per-player and per-leg information needed by the scorer.

### F-02 — Leg-bet scoring and expected value — P1

- **Type:** Feature
- **Classification:** Missing feature
- **Finding:** There is no representation of the per-colour 5/3/2 tile stacks,
  ticket ownership, or leg scoring (printed value for 1st, +1 for 2nd, −1
  otherwise).
- **Impact:** Action 1 ("take a leg-betting tile") cannot be valued.
- **Next step:** Model ticket stacks and ownership, implement the leg scoring
  table, and expose `p1`/`p2`-based EV.

### F-03 — Pyramid-ticket scoring — P1

- **Type:** Feature
- **Classification:** Missing feature
- **Finding:** Pyramid tickets are not tracked even though each is worth +1 EP at
  the next leg scoring, and taking one is the only action with a guaranteed
  positive immediate payout.
- **Impact:** Action 3 ("take a pyramid ticket") cannot be valued correctly.
- **Next step:** Track tickets per player and per leg; add the guaranteed +1 EP
  to the action evaluation.

### F-04 — Overall winner/loser decks and scoring — P1

- **Type:** Feature
- **Classification:** Missing feature
- **Finding:** There is no representation of race-betting cards, the two
  face-down decks, their placement order, or end-of-game scoring (8/5/3/2/1 for
  correct cards in order, −1 for wrong cards).
- **Impact:** Action 4 ("bet on the overall winner or loser") cannot be valued.
- **Next step:** Model each player's five cards, deck order, and the winner/loser
  scoring pass.

### F-05 — Desert income and bet interaction — P1

- **Type:** Feature
- **Classification:** Missing feature
- **Finding:** `evaluate_desert_placements` returns hit counts, not EP, and does
  not combine the +1 EP per landing with the changed leg/overall bet values.
- **Impact:** Action 2 ("place or move a desert tile") cannot be compared on a
  coin basis with the other actions.
- **Next step:** Add owner/EP accounting and evaluate the tile's effect on the
  full expected payout, not only on hit frequency.

### F-06 — Moving an existing desert tile / one-tile constraint — P2

- **Type:** Feature
- **Classification:** Missing feature
- **Finding:** The model has desert tile types but no owner and no "this player
  already has a tile on the board" state. It therefore cannot represent the
  "move your existing tile" form of action 2.
- **Impact:** Part of a legal action is missing.
- **Next step:** Add tile ownership and evaluate both placement and relocation.

### F-07 — Race-end-aware and conditional simulation — P1

- **Type:** Feature
- **Classification:** Missing feature (endgame correctness risk)
- **Finding:** `simulate_outcomes` always simulates a complete five-die leg. It
  does not stop when the race ends, does not model a partial final leg, and does
  not condition on which dice remain in the pyramid.
- **Impact:** Results are valid only for a complete leg. Using them for the final
  leg, or for a mid-leg decision, can produce invalid probabilities.
- **Next step:** Add a race-finished/partial-leg mode and a conditional
  simulation seeded with the remaining dice.

### F-08 — Input and invariant validation — P2

- **Type:** Feature
- **Classification:** Missing feature (robustness; not wrong behaviour in legal
  play)
- **Finding:** `move_unit` accepts any `steps >= 1`, although the game only
  produces 1–3. The desert-effect precondition checks only the direction of the
  effect, not the full placement invariant (no adjacent desert in either
  direction, never space 1).
- **Impact:** Invalid states can be constructed and simulated. Normal legal play
  is unaffected.
- **Next step:** Validate the dice range and the full desert invariant, or return
  a `Result` instead of panicking.

Suggested order
---------------
Steps 1 and 6 are complete on branch `fix/rust-core-bugs`; B-03 is deliberately
out of scope (see its status above).

1. **Fix B-01 and B-02** — the current results can be wrong even for the
   implemented movement-only subset.
2. **Build F-01** — the shared player/leg/action state is the foundation for all
   coin calculations.
3. **Add F-02, F-03, F-04, F-05** — score the four actions and their expected
   payouts.
4. **Add F-07** — make the evaluator valid for the final leg and mid-leg
   decisions.
5. **Finish B-03, B-04, F-06, F-08** — completeness and robustness.
6. **Clean up B-05** — cosmetic.
