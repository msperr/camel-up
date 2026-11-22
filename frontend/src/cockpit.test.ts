import { beforeEach, describe, expect, it, vi } from "vitest";
import { createRaceState, type RaceState } from "./camelRace";
import {
  buildCockpitRequest,
  fetchCockpitEvaluation,
  type CockpitResponse,
  type ThrownDie,
} from "./cockpit";
import { createLegBetState } from "./legBets";

const dice: readonly ThrownDie[] = [
  { color: "white", value: 2 },
  { color: "yellow", value: 3 },
];

const responseBody: CockpitResponse = {
  dice: { expectedCoins: 1 },
  legBet: {
    camels: [{ color: "white", expectedCoins: 2.5, roundWinProbability: 0.6 }],
  },
};

beforeEach(() => {
  vi.mocked(fetch).mockReset();
});

describe("buildCockpitRequest", () => {
  it("maps board, dice and remaining leg-bet tiles onto the API request", () => {
    const race: RaceState = {
      stacks: { 2: ["white"], 3: ["yellow", "orange"] },
      finished: null,
    };
    const legBets = { ...createLegBetState(), yellow: 2 };

    expect(buildCockpitRequest(dice, race, legBets)).toEqual({
      stacks: { "2": ["white"], "3": ["yellow", "orange"] },
      finished: null,
      thrownDice: ["white", "yellow"],
      availableLegBets: { white: 5, yellow: 3, orange: 5, green: 5, blue: 5 },
    });
  });

  it("sends a finished camel unit separately from the track", () => {
    const race: RaceState = { stacks: { 16: ["blue"] }, finished: ["white"] };

    expect(buildCockpitRequest([], race, createLegBetState()).finished).toEqual(["white"]);
  });

  it("sends an empty board for the initial position", () => {
    expect(buildCockpitRequest([], createRaceState(), createLegBetState())).toEqual({
      stacks: {},
      finished: null,
      thrownDice: [],
      availableLegBets: { white: 5, yellow: 5, orange: 5, green: 5, blue: 5 },
    });
  });
});

describe("fetchCockpitEvaluation", () => {
  it("posts the request and returns the evaluation", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => responseBody,
    } as Response);
    const request = buildCockpitRequest(dice, createRaceState(), createLegBetState());

    await expect(fetchCockpitEvaluation(request)).resolves.toEqual(responseBody);
    expect(fetch).toHaveBeenCalledWith(
      "/api/cockpit/actions",
      expect.objectContaining({
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(request),
      }),
    );
  });

  it("passes the abort signal through", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => responseBody,
    } as Response);
    const controller = new AbortController();

    await fetchCockpitEvaluation(
      buildCockpitRequest([], createRaceState(), createLegBetState()),
      controller.signal,
    );

    expect(vi.mocked(fetch).mock.calls[0][1]).toMatchObject({ signal: controller.signal });
  });

  it("reports a failed response without backend startup instructions", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({ ok: false, status: 500 } as Response);

    await expect(
      fetchCockpitEvaluation(buildCockpitRequest([], createRaceState(), createLegBetState())),
    ).rejects.toThrow(/Cockpit evaluation rejected \(HTTP 500\)/);
  });

  it("surfaces the backend's validation message", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({ error: "space 0 must be in 1..=16" }),
    } as Response);

    await expect(
      fetchCockpitEvaluation(buildCockpitRequest([], createRaceState(), createLegBetState())),
    ).rejects.toThrow(/HTTP 400\): space 0 must be in 1\.\.=16/);
  });

  it("wraps a network failure with backend startup instructions", async () => {
    vi.mocked(fetch).mockRejectedValueOnce(new TypeError("Failed to fetch"));

    await expect(
      fetchCockpitEvaluation(buildCockpitRequest([], createRaceState(), createLegBetState())),
    ).rejects.toThrow(/start the Rust backend with 'cd rust && cargo run'/);
  });

  it("does not wrap an aborted request", async () => {
    const abortError = new Error("aborted");
    vi.mocked(fetch).mockRejectedValueOnce(abortError);
    const controller = new AbortController();
    controller.abort();

    await expect(
      fetchCockpitEvaluation(
        buildCockpitRequest([], createRaceState(), createLegBetState()),
        controller.signal,
      ),
    ).rejects.toBe(abortError);
  });
});
