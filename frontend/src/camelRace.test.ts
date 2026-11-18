import { describe, expect, it } from "vitest";
import { applyDie, createRaceState, hasCamels, isRaceFinished } from "./camelRace";

describe("camel race state", () => {
  it("starts with no camels on the field", () => {
    const state = createRaceState();

    expect(state.stacks).toEqual({});
    expect(state.finished).toBeNull();
    expect(hasCamels(state)).toBe(false);
    expect(isRaceFinished(state)).toBe(false);
  });

  it("places a new camel on the space shown by its die", () => {
    expect(applyDie(createRaceState(), "white", 1).stacks).toEqual({ 1: ["white"] });
    expect(applyDie(createRaceState(), "blue", 3).stacks).toEqual({ 3: ["blue"] });
  });

  it("puts a new camel on top of a camel already on that space", () => {
    const state = applyDie(applyDie(createRaceState(), "white", 1), "yellow", 1);

    expect(state.stacks).toEqual({ 1: ["white", "yellow"] });
  });

  it("moves a camel and every camel above it, leaving lower camels behind", () => {
    const state = {
      stacks: { 1: ["blue", "white", "yellow"] },
      finished: null,
    } as const;

    expect(applyDie(state, "white", 2).stacks).toEqual({
      1: ["blue"],
      3: ["white", "yellow"],
    });
  });

  it("keeps the order of a carried stack when it lands on another stack", () => {
    const state = {
      stacks: { 2: ["white", "yellow", "orange"], 5: ["green"] },
      finished: null,
    } as const;

    expect(applyDie(state, "white", 3).stacks).toEqual({
      5: ["green", "white", "yellow", "orange"],
    });
  });

  it("does not finish when a camel lands exactly on space 16", () => {
    const state = {
      stacks: { 15: ["white"] },
      finished: null,
    } as const;
    const next = applyDie(state, "white", 1);

    expect(next.stacks).toEqual({ 16: ["white"] });
    expect(next.finished).toBeNull();
    expect(isRaceFinished(next)).toBe(false);
  });

  it("sends a camel unit past the finish line into the overflow area", () => {
    const state = {
      stacks: { 16: ["white", "yellow"] },
      finished: null,
    } as const;
    const next = applyDie(state, "white", 1);

    expect(next.stacks).toEqual({});
    expect(next.finished).toEqual(["white", "yellow"]);
    expect(isRaceFinished(next)).toBe(true);
    expect(hasCamels(next)).toBe(true);
  });

  it("does not move camels once the race has finished", () => {
    const finishedState = applyDie({ stacks: { 16: ["white"] }, finished: null }, "white", 2);

    expect(applyDie(finishedState, "yellow", 1)).toBe(finishedState);
  });

  it("clears all camels back to the initial state", () => {
    const played = applyDie(createRaceState(), "white", 1);

    expect(createRaceState()).toEqual({ stacks: {}, finished: null });
    expect(played.stacks).not.toEqual({});
  });
});
