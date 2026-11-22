/**
 * Local, presentation-level camel state for the board UI.
 *
 * The Rust crate owns the authoritative rules model. This module only tracks
 * where the pieces should be drawn while dice are entered by hand; it follows
 * the documented camel-unit movement and stacking rules (RULES.md, section 2).
 * The backend remains authoritative for ranking and expected coins.
 */

import { desertAt, type PlacedDesert } from "./desertTiles";

export type CamelColor = "white" | "yellow" | "orange" | "green" | "blue";
export type DieValue = 1 | 2 | 3;

export const CAMEL_COLORS: readonly CamelColor[] = ["white", "yellow", "orange", "green", "blue"];

/** Race track fields are numbered 1..16; a move past 16 ends the race. */
export const LAST_SPACE = 16;

/** Camels in a stack are ordered from bottom to top. */
export type CamelStack = readonly CamelColor[];

export type RaceState = {
  readonly stacks: Readonly<Record<number, CamelStack>>;
  /** The camel unit that crossed the finish line, or null while racing. */
  readonly finished: CamelStack | null;
};

export const createRaceState = (): RaceState => ({ stacks: {}, finished: null });

const findCamel = (
  stacks: RaceState["stacks"],
  color: CamelColor,
): { space: number; index: number } | null => {
  for (const [key, stack] of Object.entries(stacks)) {
    const index = stack.indexOf(color);

    if (index !== -1) {
      return { space: Number(key), index };
    }
  }

  return null;
};

const addToStack = (
  stacks: RaceState["stacks"],
  space: number,
  unit: CamelStack,
): RaceState["stacks"] => ({
  ...stacks,
  [space]: [...(stacks[space] ?? []), ...unit],
});

const prependToStack = (
  stacks: RaceState["stacks"],
  space: number,
  unit: CamelStack,
): RaceState["stacks"] => ({
  ...stacks,
  [space]: [...unit, ...(stacks[space] ?? [])],
});

/**
 * Apply one thrown die to the camel positions with desert tiles on the board.
 *
 * A new camel appears on the die's space. An existing camel carries every
 * camel above it; camels below stay behind. A unit ending its move on a tile
 * then takes one extra step: oasis lands on top in the space forward, mirage
 * goes underneath in the space backward. A tile on space 16 sends the unit
 * over the finish line. Effects never chain, so the extra step ignores any
 * tile on its destination space.
 *
 * Coin income for a tile hit is calculated by the backend, not here.
 */
export const applyDieWithDeserts = (
  state: RaceState,
  color: CamelColor,
  value: DieValue,
  deserts: readonly PlacedDesert[],
): RaceState => {
  if (state.finished !== null) {
    return state;
  }

  const found = findCamel(state.stacks, color);

  if (found === null) {
    return { stacks: addToStack(state.stacks, value, [color]), finished: null };
  }

  const source = state.stacks[found.space];
  const movingUnit = source.slice(found.index);
  const remaining = source.slice(0, found.index);
  const stacks = { ...state.stacks };

  if (remaining.length > 0) {
    stacks[found.space] = remaining;
  } else {
    delete stacks[found.space];
  }

  const destination = found.space + value;
  const tile = desertAt(deserts, destination);

  if (tile === undefined) {
    if (destination > LAST_SPACE) {
      return { stacks, finished: movingUnit };
    }
    return { stacks: addToStack(stacks, destination, movingUnit), finished: null };
  }

  if (tile.kind === "oasis") {
    // The extra forward step crosses the line when the tile sits on space 16.
    if (destination >= LAST_SPACE) {
      return { stacks, finished: movingUnit };
    }
    return { stacks: addToStack(stacks, destination + 1, movingUnit), finished: null };
  }

  return { stacks: prependToStack(stacks, destination - 1, movingUnit), finished: null };
};

/** Apply one thrown die on a board with no desert tiles. */
export const applyDie = (state: RaceState, color: CamelColor, value: DieValue): RaceState =>
  applyDieWithDeserts(state, color, value, []);

export const isRaceFinished = (state: RaceState): boolean => state.finished !== null;

export const hasCamels = (state: RaceState): boolean =>
  state.finished !== null || Object.values(state.stacks).some((stack) => stack.length > 0);
