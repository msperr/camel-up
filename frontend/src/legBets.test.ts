import { describe, expect, it } from "vitest";
import { CAMEL_COLORS } from "./camelRace";
import {
  LEG_BET_VALUES,
  createLegBetState,
  getLegBetValues,
  hasTakenLegBet,
  takeLegBet,
} from "./legBets";

describe("leg-bet tile state", () => {
  it("starts with all three tiles available for every camel", () => {
    const state = createLegBetState();

    expect(LEG_BET_VALUES).toEqual([5, 3, 2]);
    for (const color of CAMEL_COLORS) {
      expect(state[color]).toBe(3);
    }
    expect(hasTakenLegBet(state)).toBe(false);
  });

  it("maps the remaining tile count to the available tile values", () => {
    expect(getLegBetValues(3)).toEqual([5, 3, 2]);
    expect(getLegBetValues(2)).toEqual([3, 2]);
    expect(getLegBetValues(1)).toEqual([2]);
    expect(getLegBetValues(0)).toEqual([]);
    expect(getLegBetValues(-1)).toEqual([]);
    expect(getLegBetValues(4)).toEqual([5, 3, 2]);
  });

  it("takes tiles from the top: 5 first, then 3, then 2", () => {
    let state = createLegBetState();

    expect(getLegBetValues(state.white)).toEqual([5, 3, 2]);
    state = takeLegBet(state, "white");
    expect(state.white).toBe(2);
    expect(getLegBetValues(state.white)).toEqual([3, 2]);

    state = takeLegBet(state, "white");
    expect(state.white).toBe(1);
    expect(getLegBetValues(state.white)).toEqual([2]);

    state = takeLegBet(state, "white");
    expect(state.white).toBe(0);
    expect(getLegBetValues(state.white)).toEqual([]);
  });

  it("leaves an exhausted stack empty and unchanged", () => {
    const exhausted = [1, 2, 3].reduce((state) => takeLegBet(state, "blue"), createLegBetState());

    expect(exhausted.blue).toBe(0);
    expect(takeLegBet(exhausted, "blue")).toBe(exhausted);
  });

  it("only changes the camel whose tile was taken", () => {
    const state = createLegBetState();
    const next = takeLegBet(state, "orange");

    expect(next.orange).toBe(2);
    expect(next.white).toBe(3);
    expect(next.yellow).toBe(3);
    expect(next.green).toBe(3);
    expect(next.blue).toBe(3);
    expect(state.orange).toBe(3);
    expect(hasTakenLegBet(next)).toBe(true);
  });
});
