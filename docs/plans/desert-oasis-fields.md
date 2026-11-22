# Desert and oasis fields — design record and implementation plan

Status: approved, 2026-11-22. This is the handoff contract for the implementation PRs listed below.

## Goal

Let the player place desert tiles on the race track, see the best oasis/mirage placements in the player cockpit, and have those tiles affect camel movement. Multiple tiles may be on the board at once, simulating different players placing their own tiles.

## Decisions

- **Expected coins means direct income only**: 1 coin per camel unit that ends a move on the tile during the rest of the current leg. Positional effects on leg/overall bet values are not modelled; ROADMAP F-05 stays partially open.
- **Multiple independent tiles** are allowed. There is no per-player supply, ownership, or turn tracking. Oasis and mirage are always available.
- **Tiles are removed on Next round and Clear.** They do not persist across legs.
- **Artwork is stylised SVG**, not copied published art: a square sandy cardboard tile with a dark border, a palm-and-water oasis face, a dune/heat-haze mirage face, and a small coloured badge so simultaneous tiles are distinguishable.
- **Placement is disabled** once the fifth die of the leg has been thrown or the race is finished.

## Rules basis

From `RULES.md` (2014 game only):

- §6: a tile may be placed only on an empty space (no camels, no other tile), never on space 1, and never adjacent to another tile. Either face, oasis or mirage, may be chosen.
- §6: a camel unit ending its move on a tile earns its owner 1 coin, then moves one space forward (oasis, landing on top) or one space backward (mirage, going underneath). Effects do not chain because adjacent tiles are illegal. An effect can cross the finish line.
- §9: at the end of a leg, desert tiles are returned to their owners and all dice go back into the pyramid. This is why placement stops when the fifth die resolves, and why Next round clears tiles.

Additional constraint for this feature: spaces **behind the rearmost camel** are not offered, even where the rules would not explicitly forbid them (`State::evaluate_desert_placements` already applies this filter).

## Frozen interface contract

All additions are backwards-compatible: request fields use `#[serde(default)]`, and the response gains a new field.

Request — `POST /api/cockpit/actions`:

```json
{
  "stacks": { "2": ["white"], "5": ["yellow", "orange"], "9": ["green"], "12": ["blue"] },
  "finished": null,
  "thrownDice": ["white"],
  "availableLegBets": { "white": 5, "yellow": 5, "orange": 5, "green": 5, "blue": 5 },
  "deserts": [
    { "space": 8, "kind": "oasis" },
    { "space": 11, "kind": "mirage" }
  ]
}
```

Response addition:

```json
{
  "dice": { "expectedCoins": 1.0 },
  "legBet": { "camels": [] },
  "desert": {
    "options": [
      { "kind": "mirage", "space": 10, "expectedCoins": 1.25 },
      { "kind": "oasis", "space": 4, "expectedCoins": 0.75 }
    ]
  }
}
```

`options` is sorted by `expectedCoins` descending, then `space` ascending, then oasis before mirage. The frontend renders at most the first four. The array is empty at leg end, race finish, or when no legal space exists.

## Legal placement

A space is legal when all of the following hold:

1. `2..=16` (never space 1).
2. No camels on the space.
3. No desert tile on the space.
4. Neither neighbouring track space contains a tile.
5. The space is at or ahead of the rearmost camel; with no camels on the board, no option is offered.

The same rules apply in the Rust evaluator and in the frontend click-target helper so that clickable fields and cockpit options agree.

## Expected coins computation

- Rust owns the calculation; the frontend only renders the returned numbers.
- For each legal `(space, kind)`, add the hypothetical tile to a clone of the current state, simulate the remainder of the leg with `simulate_outcomes(&already_moved)`, and divide the hit count for that space by the number of simulated permutations × die-roll products. Existing tiles remain in the simulation.
- Return no options when `thrownDice.len() == 5`, the race is finished, or the board has no camels.
- Partially filled boards (some camels not yet placed) use a small fixed-seed sample of completed boards, evaluate candidates on each sample, and average; candidates illegal in a sample are skipped for that sample. Results must be deterministic for the same snapshot.

## Work breakdown

Each PR gets its own branch and worktree from an up-to-date `main` (`AGENTS.md`: one task = one worktree + one branch). Merge sequentially; clean up the worktree and branch after each merge.

### PR 0 — this document (docs only)

- Branch `docs/desert-oasis-plan`, worktree `.worktrees/desert-oasis-plan`.
- Add this file and a status link from `ROADMAP.md` F-05.

### PR 1 — Rust backend

- Branch `feat/desert-backend-api`, worktree `.worktrees/desert-backend-api`.
- Files: `rust/src/cockpit.rs`, `rust/src/state.rs`, `rust/src/lib.rs`, `rust/tests/cockpit.rs`.
- Add `deserts` to `CockpitRequest`, validate spaces `2..=16`, no duplicates, no camel/adjacent-tile conflicts.
- Add the expected-income helper next to `State::evaluate_desert_placements`.
- Add `desert.options` to `CockpitResponse`, sorted as specified.
- Tests: deserialisation, validation, legal filtering, exact expected coins on a small deterministic board, ordering, and empty results at leg end/finish.
- Additive only: the current frontend must keep working after this merge.
- Verify: `cargo check && cargo test`.

### PR 2 — Frontend model layer

- Branch `feat/desert-frontend-model`, worktree `.worktrees/desert-frontend-model`.
- Files: new `frontend/src/desertTiles.ts`, `frontend/src/camelRace.ts`, `frontend/src/cockpit.ts`.
- Types `DesertKind = "oasis" | "mirage"`, `PlacedDesert = { space, kind, badge }`; legal-space helper; SVG tile geometry/art helpers.
- Desert-aware movement: oasis moves the unit one space forward and on top; mirage one space backward and underneath; oasis on space 16 finishes; no chained effects.
- Extend cockpit types and request building to include placed tiles.
- Keep existing callers green: add `applyDieWithDeserts(...)` and leave `applyDie(...)` as a wrapper with no tiles; add a new request-builder entry point or optional argument rather than changing the existing signature.
- Verify: frontend typecheck + tests.

### PR 3 — Frontend UI

- Branch `feat/desert-ui`, worktree `.worktrees/desert-ui`.
- Files: `frontend/src/App.tsx`, `frontend/src/Cockpit.tsx`, `frontend/src/styles.css` and affected tests.
- Make legal space groups keyboard-accessible SVG buttons with a small `+` icon; disabled fields have no icon and no handler.
- Add an oasis/mirage selection dialog reusing the existing focus trap, Escape handling, focus restoration, and `inert` page behaviour.
- Render tiles flat on their space behind camel pieces; clear on Next round/Clear.
- Replace the cockpit desert placeholder with up to four sorted rows: tile kind, space, expected coins.
- Verify: `cargo test` plus `npm run typecheck`, `npm run lint`, `npm test`, `npm run build`.

## Out of scope

- Player identity, money, turns, and one-action-per-turn enforcement.
- Moving or flipping an already placed tile; tiles are simply added and cleared with the leg.
- Full bet-interaction EV for deserts (ROADMAP F-05 remains open beyond direct income).
- Copyrighted reproductions of the physical tile artwork.
