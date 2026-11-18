import { CAMEL_COLORS, type CamelColor } from "./camelRace";

/**
 * Local, presentation-level state for the leg-bet tile stacks above the board.
 *
 * The Rust crate owns the authoritative rules model. This module only tracks
 * which tiles are still visible in each colour's 5/3/2 stack while the player
 * enters dice by hand; ownership and scoring are not modelled here.
 */

/** Tiles are taken from the top of the stack: 5 first, then 3, then 2. */
export const LEG_BET_VALUES = [5, 3, 2] as const;
export type LegBetValue = (typeof LEG_BET_VALUES)[number];

/** How many tiles are still available in each camel's stack. */
export type LegBetState = Readonly<Record<CamelColor, number>>;

export const createLegBetState = (): LegBetState =>
  Object.fromEntries(CAMEL_COLORS.map((color) => [color, LEG_BET_VALUES.length])) as LegBetState;

/** The tile that is currently fully visible and available for that camel. */
export const getTopLegBetValue = (remaining: number): LegBetValue | null =>
  remaining > 0 ? LEG_BET_VALUES[LEG_BET_VALUES.length - remaining] : null;

/** Take the top tile of one stack. An exhausted stack stays exhausted. */
export const takeLegBet = (state: LegBetState, color: CamelColor): LegBetState => {
  const remaining = state[color];

  if (remaining === 0) {
    return state;
  }

  return { ...state, [color]: remaining - 1 };
};

export const hasTakenLegBet = (state: LegBetState): boolean =>
  CAMEL_COLORS.some((color) => state[color] < LEG_BET_VALUES.length);
