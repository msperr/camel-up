/**
 * Presentation-level desert tile state, placement rules and board geometry.
 *
 * The Rust crate owns the authoritative rules model and the expected coins for
 * each placement. This module only mirrors the placement restrictions from
 * RULES.md section 6 so clickable fields and cockpit options agree, and holds
 * the local geometry/art constants the board UI draws.
 */

import { BOARD_SPACES, FINISH_AFTER_SPACE, SPACE_WIDTH } from "./boardLayout";
import type { RaceState } from "./camelRace";

/** Space 1 is never a legal desert placement. */
export const FIRST_DESERT_SPACE = 2;
export const LAST_DESERT_SPACE = FINISH_AFTER_SPACE;

export type DesertKind = "oasis" | "mirage";

export const DESERT_KINDS: readonly DesertKind[] = ["oasis", "mirage"];

/**
 * Badges distinguish tiles placed at the same time. There are eight colours so
 * every non-adjacent set of tiles on spaces 2..16 can have a distinct badge.
 */
export const DESERT_BADGES = [
  "amber",
  "teal",
  "violet",
  "rose",
  "sky",
  "lime",
  "coral",
  "indigo",
] as const;
export type DesertBadge = (typeof DESERT_BADGES)[number];

/** The part of a tile that is part of the API snapshot. */
export type DesertPlacement = {
  readonly space: number;
  readonly kind: DesertKind;
};

/** A tile on the local board, with the badge used to tell tiles apart. */
export type PlacedDesert = DesertPlacement & {
  readonly badge: DesertBadge;
};

/** Find the tile on a space, if any. */
export const desertAt = (
  deserts: readonly DesertPlacement[],
  space: number,
): DesertPlacement | undefined => deserts.find((desert) => desert.space === space);

const rearmostCamelSpace = (race: RaceState): number | null => {
  const occupied = Object.entries(race.stacks)
    .filter(([, stack]) => stack.length > 0)
    .map(([space]) => Number(space));

  return occupied.length === 0 ? null : Math.min(...occupied);
};

/**
 * Whether a tile may be placed on `space` (RULES.md section 6): never space 1,
 * never an occupied space, never next to another tile, and never behind the
 * rearmost camel. A finished or empty board has no legal placements.
 */
export const isLegalDesertPlacement = (
  space: number,
  race: RaceState,
  deserts: readonly DesertPlacement[],
): boolean => {
  if (race.finished !== null) {
    return false;
  }
  if (!Number.isInteger(space) || space < FIRST_DESERT_SPACE || space > LAST_DESERT_SPACE) {
    return false;
  }

  const rearmost = rearmostCamelSpace(race);
  if (rearmost === null || space < rearmost) {
    return false;
  }
  if ((race.stacks[space]?.length ?? 0) > 0) {
    return false;
  }
  if (desertAt(deserts, space) !== undefined) {
    return false;
  }
  if (desertAt(deserts, space - 1) !== undefined || desertAt(deserts, space + 1) !== undefined) {
    return false;
  }

  return true;
};

/** Legal placements for both faces, ordered by space then face. */
export const getLegalDesertPlacements = (
  race: RaceState,
  deserts: readonly DesertPlacement[],
): readonly DesertPlacement[] => {
  const spaces: number[] = [];
  for (let space = FIRST_DESERT_SPACE; space <= LAST_DESERT_SPACE; space += 1) {
    spaces.push(space);
  }

  return spaces.flatMap((space) =>
    isLegalDesertPlacement(space, race, deserts)
      ? DESERT_KINDS.map((kind) => ({ space, kind }))
      : [],
  );
};

/** The first badge not yet in use, cycling deterministically if all are used. */
export const nextDesertBadge = (existing: readonly PlacedDesert[]): DesertBadge => {
  const used = new Set(existing.map((desert) => desert.badge));
  const free = DESERT_BADGES.find((badge) => !used.has(badge));

  return free ?? DESERT_BADGES[existing.length % DESERT_BADGES.length];
};

/** Add a tile to the local board with a badge that distinguishes it. */
export const createPlacedDesert = (
  space: number,
  kind: DesertKind,
  existing: readonly PlacedDesert[],
): PlacedDesert => ({ space, kind, badge: nextDesertBadge(existing) });

/** Tiles are drawn inside their 200px field with a small margin. */
export const DESERT_TILE_SIZE = SPACE_WIDTH * 0.72;
export const DESERT_BADGE_INSET = 14;
export const DESERT_BADGE_RADIUS = 11;

export type DesertTilePlacement = {
  /** Centre of the tile in board viewBox coordinates. */
  readonly x: number;
  readonly y: number;
  readonly size: number;
  readonly badge: {
    readonly x: number;
    readonly y: number;
    readonly radius: number;
  };
};

/** Where to draw a tile and its badge; `null` for a non-track space. */
export const getDesertTilePlacement = (space: number): DesertTilePlacement | null => {
  const field = BOARD_SPACES.find((item) => item.space === space);
  if (!field) {
    return null;
  }

  const half = DESERT_TILE_SIZE / 2;
  return {
    x: field.x,
    y: field.y,
    size: DESERT_TILE_SIZE,
    badge: {
      x: field.x + half - DESERT_BADGE_INSET,
      y: field.y - half + DESERT_BADGE_INSET,
      radius: DESERT_BADGE_RADIUS,
    },
  };
};

/** Art is drawn in a 100 x 100 local square, then scaled onto the tile. */
export const DESERT_ART_VIEWBOX = 100;

export const DESERT_ART = {
  oasis: {
    water: "M 10 70 Q 30 60 50 70 T 90 70 L 90 88 Q 50 80 10 88 Z",
    trunk: "M 52 70 Q 47 48 55 26",
    fronds:
      "M 55 26 Q 38 16 26 26 M 55 26 Q 45 10 32 10 M 55 26 Q 66 10 78 16 M 55 26 Q 72 22 82 34",
  },
  mirage: {
    dune: "M 6 80 Q 28 48 48 66 Q 68 42 94 80 Z",
    haze: "M 12 42 Q 32 34 52 42 T 88 42 M 18 54 Q 38 48 58 54 T 84 54",
  },
} as const;

/** SVG transform mapping the local art square onto a placed tile. */
export const getDesertArtTransform = (placement: DesertTilePlacement): string =>
  `translate(${placement.x - placement.size / 2} ${placement.y - placement.size / 2}) scale(${placement.size / DESERT_ART_VIEWBOX})`;
