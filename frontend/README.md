# Camel Up Frontend

Single-page TypeScript frontend for the Camel Up decision tracker.

## Scope

- One page with the Camel Up board centred on screen.
- The board is a hand-built, seamless square 5 x 5 SVG with a square frame. The
  16 dark Camel Up fields fill the outer ring without gaps; field 1 is at the
  top middle, field 16 directly to its left, and the checkered finish line runs
  along their shared edge. The light central 3 x 3 panel contains a centred
  3 x 1 dice tray with five empty slots. Only the twelve non-corner fields
  carry small numbers at their inner edge; the four corner fields are blank. It
  uses no scanned board or component artwork.
- The dice tray starts empty. Its plus button opens an add-die dialog with the
  colours that have not been thrown yet and the values 1, 2, or 3. A random icon
  fills both fields, OK appends the die to the next slot in throw order, and
  Cancel, Escape, or the backdrop closes the dialog. The tray holds at most five
  dice, one per colour. Clear removes all dice and camels; Next round is enabled
  only once all five dice are on the field and clears the dice while the camels
  stay where they are. All UI text is English.
- Side-view camel meeples in the dice colours appear at the die value when a die
  is thrown. A camel already on that field stays below the newcomer; moving a
  camel carries the camels above it and leaves those below behind. Five tight
  camels fit in one field, centred without changing size. A camel that passes
  space 16 is drawn above the finish line and locks the round: the plus button
  and Next round are disabled until Clear resets the board.
- Above the track, five stacks of square leg-bet cards show each camel's 5, 3,
  and 2 tiles. Only the fully visible top tile is clickable; taking it reveals
  the next tile, and an exhausted stack leaves an empty, non-clickable outline.
  Card clicks are disabled throughout the setup round (before the first leg),
  once five dice have been thrown, or when the race has finished. Clear and
  Next round restore all three tiles for every camel.
- Below the board, a read-only cockpit lists the four actions using their names
  from RULES.md. “Take a pyramid ticket and roll” shows the ticket's guaranteed
  +1 EP with no camel columns while a ticket can still be taken; it shows a dash
  during setup, once all five dice are thrown, or when the race has finished.
  “Take a leg-betting tile” is a dash whenever taking a tile is unavailable
  (setup, five dice, or race finish); in a leg it shows one cell per camel with net
  expected EP (two decimals) and the probability of winning the round (whole
  percent), ordered by expected coins, then probability, then colour; an
  exhausted stack is blank and sorted last. The desert and overall bet rows are
  placeholders. The cockpit has no controls.
- Expected values are computed by the Rust crate at `POST /api/cockpit/actions`;
  Vite proxies `/api` to `127.0.0.1:3000`. Run `cargo run` in `rust/` alongside
  `npm run dev`. Camel positions remain temporary local UI state for driving
  the SVG; the Rust crate remains the authoritative rules model. It rejects
  board/dice combinations that cannot occur, marks setup snapshots with
  `setupRound` to suppress both actions' values, marks both actions unavailable
  after five dice or at race finish, and averages over the setup stack orders
  left open by RULES.md section 3 for other callers.
- Values are cleared while a new snapshot is being evaluated, so they never
  describe a different board. A validation failure shows the backend's message;
  only a network failure shows the instruction to start the Rust backend.

The implementation follows the frontend architecture decisions in
`ROADMAP.md` (D-01 to D-07; currently on the
`docs/frontend-architecture-decisions` branch) for the TypeScript SPA, React +
Vite setup, SVG board, and static space-to-coordinate map. The local movement
in `camelRace.ts` is presentation scaffolding until the backend API (D-03 to
D-07) exists; ranking and legal actions must still come from the backend.

## Stack

- React 19 + TypeScript
- Vite
- Vitest + Testing Library
- ESLint + Prettier

## Commands

```sh
cd frontend
npm install
npm run dev          # Vite dev server
npm run build        # type-check and production build
npm run preview      # preview the production build
npm run typecheck    # TypeScript only
npm run lint
npm run format
npm run format:check
npm test             # run tests once
npm run test:watch
```

Node.js 22.12 or newer is required.

## Structure

```text
frontend/
  index.html
  src/
    App.tsx           # the one page: SVG board, dice, camels, and add-die dialog
    Cockpit.tsx       # read-only action overview below the board
    boardLayout.ts    # static 5 x 5 map and camel/stack geometry for spaces 1..16
    camelRace.ts      # local camel positions and die-driven movement
    cockpit.ts        # Rust API request/response client for the cockpit
    legBets.ts        # local 5/3/2 leg-bet tile stacks
    main.tsx
    styles.css
    test/setup.ts
```

## Intentionally not built yet

- A general API client, DTO layer, and server snapshot state; the cockpit uses
  one narrow endpoint and does not dispatch commands.
- Server snapshot state, command dispatch, revision handling, and undo.
- Expected values for the desert-tile and overall-bet actions; their rows are
  placeholders.
- Drag-and-drop, edit mode, and legal-action highlighting.

Those belong to the game, analysis, and HTTP adapter layers described in
D-03 to D-07. The board map is presentation-only; ranking and legal moves must
come from the backend.
