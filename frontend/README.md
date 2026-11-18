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
- No backend connection. Camel positions are temporary local UI state for
  driving the SVG; the Rust crate remains the authoritative rules model.

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
    boardLayout.ts    # static 5 x 5 map and camel/stack geometry for spaces 1..16
    camelRace.ts      # local camel positions and die-driven movement
    main.tsx
    styles.css
    test/setup.ts
```

## Intentionally not built yet

- API client, DTO types, and the Vite development proxy to the Rust server.
- Server snapshot state, command dispatch, revision handling, and undo.
- Analysis results, progress, caching, and action evaluation.
- Drag-and-drop, edit mode, and legal-action highlighting.

Those belong to the game, analysis, and HTTP adapter layers described in
D-03 to D-07. The board map is presentation-only; ranking and legal moves must
come from the backend.
