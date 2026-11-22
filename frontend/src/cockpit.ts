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
  "Cockpit unavailable: start the Rust backend with 'cd rust && cargo run', then reload.";

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

/** Read the backend's validation message from a non-2xx response, if present. */
const rejectionDetail = async (response: Response): Promise<string> => {
  try {
    const body: unknown = await response.json();
    if (
      typeof body === "object" &&
      body !== null &&
      "error" in body &&
      typeof body.error === "string" &&
      body.error.length > 0
    ) {
      return body.error;
    }
  } catch {
    // A non-JSON error body is still reported with its HTTP status.
  }

  return "no error details";
};

/** Ask the Rust API to evaluate the actions for one board snapshot. */
export const fetchCockpitEvaluation = async (
  request: CockpitRequest,
  signal?: AbortSignal,
): Promise<CockpitResponse> => {
  let response: Response;

  try {
    response = await fetch(COCKPIT_PATH, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
      signal,
    });
  } catch (error) {
    if (signal?.aborted) {
      throw error;
    }

    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`${BACKEND_UNAVAILABLE_MESSAGE} Details: ${detail}`, { cause: error });
  }

  // A validation failure is a reply from a healthy backend. Surface its
  // message rather than telling the user to start the server.
  if (!response.ok) {
    const detail = await rejectionDetail(response);
    throw new Error(`Cockpit evaluation rejected (HTTP ${response.status}): ${detail}`);
  }

  return (await response.json()) as CockpitResponse;
};
