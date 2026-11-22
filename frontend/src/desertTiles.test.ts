import { describe, expect, it } from "vitest";
import { BOARD_SPACES, SPACE_WIDTH } from "./boardLayout";
import { createRaceState, type RaceState } from "./camelRace";
import {
  createPlacedDesert,
  DESERT_ART,
  DESERT_PLUS_ARM,
  DESERT_PLUS_INSET,
  DESERT_TILE_SIZE,
  getDesertArtTransform,
  getDesertTilePlacement,
  getLegalDesertPlacements,
  isLegalDesertPlacement,
  type DesertKind,
  type DesertPlacement,
} from "./desertTiles";

const raceWith = (stacks: RaceState["stacks"]): RaceState => ({ stacks, finished: null });
const tile = (space: number, kind: DesertKind = "mirage"): DesertPlacement => ({ space, kind });

describe("legal desert placement", () => {
  it("allows an empty space at or ahead of the rearmost camel", () => {
    const race = raceWith({ 5: ["white"] });

    expect(isLegalDesertPlacement(5, race, [])).toBe(false);
    expect(isLegalDesertPlacement(6, race, [])).toBe(true);
    expect(isLegalDesertPlacement(16, race, [])).toBe(true);
  });

  it("forbids space 1, spaces behind the camels and off-board spaces", () => {
    const race = raceWith({ 5: ["white"] });

    expect(isLegalDesertPlacement(1, race, [])).toBe(false);
    expect(isLegalDesertPlacement(2, race, [])).toBe(false);
    expect(isLegalDesertPlacement(4, race, [])).toBe(false);
    expect(isLegalDesertPlacement(17, race, [])).toBe(false);
    expect(isLegalDesertPlacement(1.5, race, [])).toBe(false);
  });

  it("forbids occupied and adjacent-to-tile spaces", () => {
    const race: RaceState = { stacks: { 5: ["white"], 9: ["blue"] }, finished: null };
    const deserts = [tile(7)];

    expect(isLegalDesertPlacement(7, race, deserts)).toBe(false);
    expect(isLegalDesertPlacement(6, race, deserts)).toBe(false);
    expect(isLegalDesertPlacement(8, race, deserts)).toBe(false);
    expect(isLegalDesertPlacement(5, race, deserts)).toBe(false);
    expect(isLegalDesertPlacement(10, race, deserts)).toBe(true);
  });

  it("has no legal placements on an empty or finished board", () => {
    expect(getLegalDesertPlacements(createRaceState(), [])).toEqual([]);
    expect(getLegalDesertPlacements({ stacks: { 8: ["white"] }, finished: ["blue"] }, [])).toEqual(
      [],
    );
  });

  it("offers both faces for every legal space, ordered by space", () => {
    const race = raceWith({ 8: ["white"] });
    const placements = getLegalDesertPlacements(race, []);

    expect(placements.slice(0, 4)).toEqual([
      { space: 9, kind: "oasis" },
      { space: 9, kind: "mirage" },
      { space: 10, kind: "oasis" },
      { space: 10, kind: "mirage" },
    ]);
    expect(placements.every((placement) => isLegalDesertPlacement(placement.space, race, []))).toBe(
      true,
    );
  });
});

describe("tile geometry and art", () => {
  it("creates a tile from its space and face only", () => {
    expect(createPlacedDesert(4, "oasis")).toEqual({ space: 4, kind: "oasis" });
  });

  it("keeps the clickable-field plus small and inside the field corner", () => {
    expect(DESERT_PLUS_INSET).toBeGreaterThan(DESERT_PLUS_ARM);
    expect(DESERT_PLUS_INSET + DESERT_PLUS_ARM).toBeLessThan(SPACE_WIDTH / 2);
  });

  it("centres a tile inside its field", () => {
    const placement = getDesertTilePlacement(8);
    const field = BOARD_SPACES.find((space) => space.space === 8);

    expect(placement).not.toBeNull();
    expect(placement?.x).toBe(field?.x);
    expect(placement?.y).toBe(field?.y);
    expect(DESERT_TILE_SIZE).toBeLessThanOrEqual(SPACE_WIDTH);
    expect(getDesertTilePlacement(17)).toBeNull();
  });

  it("builds an SVG transform that scales the local art onto the tile", () => {
    const placement = getDesertTilePlacement(8);
    if (placement === null) {
      throw new Error("expected a placement for space 8");
    }

    expect(getDesertArtTransform(placement)).toContain(`scale(${DESERT_TILE_SIZE / 100})`);
    expect(DESERT_ART.oasis.water.length).toBeGreaterThan(0);
    expect(DESERT_ART.oasis.fronds.length).toBeGreaterThan(0);
    expect(DESERT_ART.mirage.dune.length).toBeGreaterThan(0);
    expect(DESERT_ART.mirage.haze.length).toBeGreaterThan(0);
  });
});
