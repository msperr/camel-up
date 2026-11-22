/**
 * Frontend client for the Rust cockpit evaluation endpoint.
 *
 * The rules model and all expected-value calculations live in the Rust crate.
 * This module only maps the board's temporary local state onto the request DTO
 * and renders the response; it contains no rules logic of its own.
 */

import { CAMEL_COLORS, type CamelColor, type DieValue, type RaceState } from "./camelRace";
import { getLegBetValues, type LegBetState } from "./legBets";

const COCKPIT_PATH = "/api/cockpit/actions";

const BACKEND_UNAVAILABLE_MESSAGE =
  "Cockpit evaluation failed: cannot reach the Rust backend at http://127.0.0.1:3000. " +
  "Start it with 'cd rust && cargo run', then reload.";

export type ThrownDie = {
  readonly color: CamelColor;
  readonly value: DieValue;
};

export type CockpitRequest = {
  readonly stacks: Readonly<Record<string, readonly CamelColor[]>>;
  readonly finished: readonly CamelColor[] | null;
  readonly thrownDice: readonly CamelColor[];
  readonly availableLegBets: Readonly<Record<CamelColor, number | null>>;
};

export type CamelOdds = {
  readonly color: CamelColor;
  /** Net expected EP of taking this colour's top leg-betting tile. */
  readonly expectedCoins: number | null;
  /** `p1`: probability of being 1st at the end of the current leg. */
  readonly roundWinProbability: number | null;
};

export type CockpitResponse = {
  readonly dice: { readonly expectedCoins: number };
  readonly legBet: { readonly camels: readonly CamelOdds[] };
};

/** Turn the current board snapshot into the JSON body the Rust API expects. */
export const buildCockpitRequest = (
  dice: readonly ThrownDie[],
  race: RaceState,
  legBets: LegBetState,
): CockpitRequest => {
  const stacks: Record<string, CamelColor[]> = {};

  for (const [space, stack] of Object.entries(race.stacks)) {
    if (stack.length > 0) {
      stacks[space] = [...stack];
    }
  }

  return {
    stacks,
    finished: race.finished === null ? null : [...race.finished],
    thrownDice: dice.map((die) => die.color),
    availableLegBets: Object.fromEntries(
      CAMEL_COLORS.map((color) => [color, getLegBetValues(legBets[color])[0] ?? null]),
    ) as Record<CamelColor, number | null>,
  };
};

/** Ask the Rust API to evaluate the actions for one board snapshot. */
export const fetchCockpitEvaluation = async (
  request: CockpitRequest,
  signal?: AbortSignal,
): Promise<CockpitResponse> => {
  try {
    const response = await fetch(COCKPIT_PATH, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
      signal,
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    return (await response.json()) as CockpitResponse;
  } catch (error) {
    if (signal?.aborted) {
      throw error;
    }

    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`${BACKEND_UNAVAILABLE_MESSAGE} Details: ${detail}`, { cause: error });
  }
};
