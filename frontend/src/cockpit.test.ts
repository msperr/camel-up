import { beforeEach, describe, expect, it, vi } from "vitest";
import { createRaceState, type RaceState } from "./camelRace";
import {
  buildCockpitRequest,
  buildCockpitRequestWithDeserts,
  fetchCockpitEvaluation,
  type CockpitResponse,
  type ThrownDie,
} from "./cockpit";
import type { PlacedDesert } from "./desertTiles";
import { createLegBetState } from "./legBets";

const dice: readonly ThrownDie[] = [
  { color: "white", value: 2 },
  { color: "yellow", value: 3 },
];

const responseBody: CockpitResponse = {
  dice: { expectedCoins: 1 },
  legBet: {
    available: true,
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

    expect(buildCockpitRequest(dice, race, legBets, false)).toEqual({
      stacks: { "2": ["white"], "3": ["yellow", "orange"] },
      finished: null,
      thrownDice: ["white", "yellow"],
      availableLegBets: { white: 5, yellow: 3, orange: 5, green: 5, blue: 5 },
      setupRound: false,
    });
  });

  it("sends a finished camel unit separately from the track", () => {
    const race: RaceState = { stacks: { 16: ["blue"] }, finished: ["white"] };

    expect(buildCockpitRequest([], race, createLegBetState(), false).finished).toEqual(["white"]);
  });

  it("sends an empty board for the initial position", () => {
    expect(buildCockpitRequest([], createRaceState(), createLegBetState(), false)).toEqual({
      stacks: {},
      finished: null,
      thrownDice: [],
      availableLegBets: { white: 5, yellow: 5, orange: 5, green: 5, blue: 5 },
      setupRound: false,
    });
  });

  it("marks the initial position as a setup round", () => {
    expect(buildCockpitRequest([], createRaceState(), createLegBetState(), true).setupRound).toBe(
      true,
    );
  });

  it("includes placed desert tiles without their local badge", () => {
    const deserts: PlacedDesert[] = [
      { space: 8, kind: "oasis", badge: "amber" },
      { space: 11, kind: "mirage", badge: "teal" },
    ];

    expect(
      buildCockpitRequestWithDeserts(dice, createRaceState(), createLegBetState(), false, deserts)
        .deserts,
    ).toEqual([
      { space: 8, kind: "oasis" },
      { space: 11, kind: "mirage" },
    ]);
  });
});

describe("fetchCockpitEvaluation", () => {
  it("posts the request and returns the evaluation", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => responseBody,
    } as Response);
    const request = buildCockpitRequest(dice, createRaceState(), createLegBetState(), false);

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

  it("returns the backend's desert recommendations", async () => {
    const body: CockpitResponse = {
      ...responseBody,
      desert: {
        options: [
          { kind: "mirage", space: 10, expectedCoins: 1.25 },
          { kind: "oasis", space: 4, expectedCoins: 0.75 },
        ],
      },
    };
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => body,
    } as Response);

    await expect(
      fetchCockpitEvaluation(
        buildCockpitRequest([], createRaceState(), createLegBetState(), false),
      ),
    ).resolves.toEqual(body);
  });

  it("passes the abort signal through", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => responseBody,
    } as Response);
    const controller = new AbortController();

    await fetchCockpitEvaluation(
      buildCockpitRequest([], createRaceState(), createLegBetState(), false),
      controller.signal,
    );

    expect(vi.mocked(fetch).mock.calls[0][1]).toMatchObject({ signal: controller.signal });
  });

  it("reports a failed response without backend startup instructions", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({ ok: false, status: 500 } as Response);

    await expect(
      fetchCockpitEvaluation(
        buildCockpitRequest([], createRaceState(), createLegBetState(), false),
      ),
    ).rejects.toThrow(/Cockpit evaluation rejected \(HTTP 500\)/);
  });

  it("surfaces the backend's validation message", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({ error: "space 0 must be in 1..=16" }),
    } as Response);

    await expect(
      fetchCockpitEvaluation(
        buildCockpitRequest([], createRaceState(), createLegBetState(), false),
      ),
    ).rejects.toThrow(/HTTP 400\): space 0 must be in 1\.\.=16/);
  });

  it("wraps a network failure with backend startup instructions", async () => {
    vi.mocked(fetch).mockRejectedValueOnce(new TypeError("Failed to fetch"));

    await expect(
      fetchCockpitEvaluation(
        buildCockpitRequest([], createRaceState(), createLegBetState(), false),
      ),
    ).rejects.toThrow(/start the Rust backend with 'cd rust && cargo run'/);
  });

  it("does not wrap an aborted request", async () => {
    const abortError = new Error("aborted");
    vi.mocked(fetch).mockRejectedValueOnce(abortError);
    const controller = new AbortController();
    controller.abort();

    await expect(
      fetchCockpitEvaluation(
        buildCockpitRequest([], createRaceState(), createLegBetState(), false),
        controller.signal,
      ),
    ).rejects.toBe(abortError);
  });
});
