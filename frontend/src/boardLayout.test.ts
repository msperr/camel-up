import { describe, expect, it } from "vitest";
import {
  BOARD_SPACES,
  BOARD_VIEWBOX,
  CELL_SIZE,
  CENTER_PANEL,
  DICE_ADD_BUTTON,
  DICE_GAP,
  DICE_ROW_Y,
  DICE_SIZE,
  DICE_TRAY,
  FINISH_AFTER_SPACE,
  FINISH_LINE,
  GRID_SIZE,
  NUMBER_EDGE_OFFSET,
  SPACE_HEIGHT,
  SPACE_WIDTH,
  START_SPACE,
  getDiceRowCenters,
  getSpaceNumberPlacement,
  isCornerSpace,
  type BoardSpace,
} from "./boardLayout";

const bySpace = new Map(BOARD_SPACES.map((space) => [space.space, space]));

const getSpace = (spaceNumber: number): BoardSpace => {
  const space = bySpace.get(spaceNumber);

  if (!space) {
    throw new Error(`Missing board space ${spaceNumber}`);
  }

  return space;
};

const getPlacement = (spaceNumber: number) => {
  const placement = getSpaceNumberPlacement(getSpace(spaceNumber));

  if (!placement) {
    throw new Error(`Missing number placement for space ${spaceNumber}`);
  }

  return placement;
};

describe("board layout", () => {
  it("uses a square 5 x 5 grid", () => {
    expect(BOARD_VIEWBOX.width).toBe(BOARD_VIEWBOX.height);
    expect(GRID_SIZE).toBe(5);
    expect(CELL_SIZE).toBe(BOARD_VIEWBOX.width / GRID_SIZE);
    expect(SPACE_WIDTH).toBe(CELL_SIZE);
    expect(SPACE_HEIGHT).toBe(CELL_SIZE);
  });

  it("defines every track space exactly once, in clockwise order", () => {
    expect(BOARD_SPACES.map((space) => space.space)).toEqual(
      Array.from({ length: 16 }, (_, index) => index + 1),
    );
  });

  it("places all 16 spaces on the outer ring", () => {
    expect(BOARD_SPACES).toHaveLength(16);

    for (const space of BOARD_SPACES) {
      const onOuterRing =
        space.column === 1 ||
        space.column === GRID_SIZE ||
        space.row === 1 ||
        space.row === GRID_SIZE;

      expect(onOuterRing, `space ${space.space}`).toBe(true);
    }
  });

  it("fills every grid cell completely, without gaps", () => {
    for (const space of BOARD_SPACES) {
      expect(space.width, `space ${space.space}`).toBe(CELL_SIZE);
      expect(space.height, `space ${space.space}`).toBe(CELL_SIZE);
      expect(space.x, `space ${space.space}`).toBe((space.column - 0.5) * CELL_SIZE);
      expect(space.y, `space ${space.space}`).toBe((space.row - 0.5) * CELL_SIZE);
      expect(space.x - space.width / 2, `space ${space.space}`).toBe(
        (space.column - 1) * CELL_SIZE,
      );
      expect(space.y - space.height / 2, `space ${space.space}`).toBe((space.row - 1) * CELL_SIZE);
    }
  });

  it("puts field 1 at the top middle and field 16 directly to its left", () => {
    const start = getSpace(START_SPACE);
    const finish = getSpace(FINISH_AFTER_SPACE);

    expect(start).toMatchObject({ column: 3, row: 1, x: 500, y: 100 });
    expect(finish).toMatchObject({ column: 2, row: 1, x: 300, y: 100 });
    expect(start.y).toBe(finish.y);
  });

  it("puts the finish line on the shared edge between fields 16 and 1", () => {
    const start = getSpace(START_SPACE);
    const finish = getSpace(FINISH_AFTER_SPACE);
    const finishRightEdge = finish.x + finish.width / 2;
    const startLeftEdge = start.x - start.width / 2;

    expect(finishRightEdge).toBe(startLeftEdge);
    expect(FINISH_LINE).toEqual({
      x: finishRightEdge,
      y: 0,
      width: 10,
      height: CELL_SIZE,
    });
  });

  it("uses the full central 3 x 3 area, touching the surrounding fields", () => {
    expect(CENTER_PANEL).toEqual({
      x: CELL_SIZE,
      y: CELL_SIZE,
      width: CELL_SIZE * 3,
      height: CELL_SIZE * 3,
    });
    expect(CENTER_PANEL.x + CENTER_PANEL.width / 2).toBe(BOARD_VIEWBOX.width / 2);
    expect(CENTER_PANEL.y + CENTER_PANEL.height / 2).toBe(BOARD_VIEWBOX.height / 2);

    for (const space of BOARD_SPACES) {
      if (isCornerSpace(space)) {
        continue;
      }

      if (space.row === 1) {
        expect(space.y + space.height / 2).toBe(CENTER_PANEL.y);
      } else if (space.column === GRID_SIZE) {
        expect(space.x - space.width / 2).toBe(CENTER_PANEL.x + CENTER_PANEL.width);
      } else if (space.row === GRID_SIZE) {
        expect(space.y - space.height / 2).toBe(CENTER_PANEL.y + CENTER_PANEL.height);
      } else {
        expect(space.x + space.width / 2).toBe(CENTER_PANEL.x);
      }
    }
  });

  it("centres a 3 x 1 dice tray in the central panel with equal space above and below", () => {
    expect(DICE_TRAY).toEqual({
      x: CENTER_PANEL.x,
      y: CENTER_PANEL.y + CENTER_PANEL.height / 3,
      width: CENTER_PANEL.width,
      height: CENTER_PANEL.height / 3,
    });

    const spaceAbove = DICE_TRAY.y - CENTER_PANEL.y;
    const spaceBelow = CENTER_PANEL.y + CENTER_PANEL.height - (DICE_TRAY.y + DICE_TRAY.height);

    expect(spaceAbove).toBe(200);
    expect(spaceBelow).toBe(200);
    expect(spaceAbove).toBe(spaceBelow);
  });

  it("centres five dice in the tray with equal side and vertical margins", () => {
    const centers = getDiceRowCenters(5);

    expect(centers).toEqual([304, 402, 500, 598, 696]);
    expect(DICE_SIZE).toBe(72);
    expect(DICE_GAP).toBe(26);
    expect(DICE_ROW_Y).toBe(DICE_TRAY.y + DICE_TRAY.height / 2);

    const rowWidth = 5 * DICE_SIZE + 4 * DICE_GAP;
    const leftMargin = centers[0] - DICE_SIZE / 2 - DICE_TRAY.x;
    const rightMargin =
      DICE_TRAY.x + DICE_TRAY.width - (centers[centers.length - 1] + DICE_SIZE / 2);
    const topMargin = DICE_ROW_Y - DICE_SIZE / 2 - DICE_TRAY.y;
    const bottomMargin = DICE_TRAY.y + DICE_TRAY.height - (DICE_ROW_Y + DICE_SIZE / 2);

    expect(rowWidth).toBe(464);
    expect(leftMargin).toBe(68);
    expect(rightMargin).toBe(68);
    expect(leftMargin).toBe(rightMargin);
    expect(topMargin).toBe(64);
    expect(bottomMargin).toBe(64);
    expect(topMargin).toBe(bottomMargin);
  });

  it("returns no dice positions for an empty row", () => {
    expect(getDiceRowCenters(0)).toEqual([]);
    expect(getDiceRowCenters(-1)).toEqual([]);
  });

  it("places the add-die button inside the tray's lower-right corner", () => {
    const centers = getDiceRowCenters(5);
    const lastDieRight = centers[centers.length - 1] + DICE_SIZE / 2;

    expect(DICE_ADD_BUTTON).toEqual({ x: 766, y: 568, size: 30 });
    expect(DICE_ADD_BUTTON.x + DICE_ADD_BUTTON.size / 2).toBeLessThanOrEqual(
      DICE_TRAY.x + DICE_TRAY.width,
    );
    expect(DICE_ADD_BUTTON.y + DICE_ADD_BUTTON.size / 2).toBeLessThanOrEqual(
      DICE_TRAY.y + DICE_TRAY.height,
    );
    expect(DICE_ADD_BUTTON.x - DICE_ADD_BUTTON.size / 2).toBeGreaterThan(lastDieRight);
    expect(DICE_ADD_BUTTON.y - DICE_ADD_BUTTON.size / 2).toBeGreaterThan(
      DICE_ROW_Y + DICE_SIZE / 2,
    );
  });

  it("leaves exactly the four corner fields without numbers", () => {
    const corners = BOARD_SPACES.filter(isCornerSpace).map((space) => space.space);
    const numbered = BOARD_SPACES.filter((space) => !isCornerSpace(space)).map(
      (space) => space.space,
    );

    expect(corners).toEqual([3, 7, 11, 15]);
    expect(numbered).toEqual([1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16]);
    expect(numbered).toHaveLength(12);

    for (const space of BOARD_SPACES) {
      if (isCornerSpace(space)) {
        expect(getSpaceNumberPlacement(space), `space ${space.space}`).toBeNull();
      } else {
        expect(getSpaceNumberPlacement(space), `space ${space.space}`).not.toBeNull();
      }
    }
  });

  it("places every number 20px from its inner edge", () => {
    for (const space of BOARD_SPACES) {
      if (isCornerSpace(space)) {
        continue;
      }

      const placement = getPlacement(space.space);

      if (space.row === 1) {
        expect(placement.y).toBe(space.y + space.height / 2 - NUMBER_EDGE_OFFSET);
      } else if (space.column === GRID_SIZE) {
        expect(placement.x).toBe(space.x - space.width / 2 + NUMBER_EDGE_OFFSET);
      } else if (space.row === GRID_SIZE) {
        expect(placement.y).toBe(space.y - space.height / 2 + NUMBER_EDGE_OFFSET);
      } else {
        expect(placement.x).toBe(space.x + space.width / 2 - NUMBER_EDGE_OFFSET);
      }
    }
  });

  it("turns every number with its top pointing outwards", () => {
    for (const space of BOARD_SPACES) {
      if (isCornerSpace(space)) {
        continue;
      }

      const placement = getPlacement(space.space);

      if (space.row === 1) {
        expect(placement.rotation).toBe(0);
      } else if (space.column === GRID_SIZE) {
        expect(placement.rotation).toBe(90);
      } else if (space.row === GRID_SIZE) {
        expect(placement.rotation).toBe(180);
      } else {
        expect(placement.rotation).toBe(-90);
      }
    }
  });

  it("uses the expected number positions", () => {
    expect(getPlacement(16)).toEqual({ x: 300, y: 180, rotation: 0 });
    expect(getPlacement(1)).toEqual({ x: 500, y: 180, rotation: 0 });
    expect(getPlacement(2)).toEqual({ x: 700, y: 180, rotation: 0 });
    expect(getPlacement(4)).toEqual({ x: 820, y: 300, rotation: 90 });
    expect(getPlacement(5)).toEqual({ x: 820, y: 500, rotation: 90 });
    expect(getPlacement(6)).toEqual({ x: 820, y: 700, rotation: 90 });
    expect(getPlacement(8)).toEqual({ x: 700, y: 820, rotation: 180 });
    expect(getPlacement(9)).toEqual({ x: 500, y: 820, rotation: 180 });
    expect(getPlacement(10)).toEqual({ x: 300, y: 820, rotation: 180 });
    expect(getPlacement(12)).toEqual({ x: 180, y: 700, rotation: -90 });
    expect(getPlacement(13)).toEqual({ x: 180, y: 500, rotation: -90 });
    expect(getPlacement(14)).toEqual({ x: 180, y: 300, rotation: -90 });
  });

  it("moves one grid step clockwise from each space to the next", () => {
    for (let spaceNumber = 1; spaceNumber <= 16; spaceNumber += 1) {
      const current = getSpace(spaceNumber);
      const next = getSpace(spaceNumber === 16 ? 1 : spaceNumber + 1);
      const gridDistance =
        Math.abs(current.column - next.column) + Math.abs(current.row - next.row);

      expect(gridDistance, `${spaceNumber} -> ${next.space}`).toBe(1);
    }
  });

  it("keeps every space inside the SVG viewBox", () => {
    for (const space of BOARD_SPACES) {
      expect(space.x - space.width / 2).toBeGreaterThanOrEqual(0);
      expect(space.y - space.height / 2).toBeGreaterThanOrEqual(0);
      expect(space.x + space.width / 2).toBeLessThanOrEqual(BOARD_VIEWBOX.width);
      expect(space.y + space.height / 2).toBeLessThanOrEqual(BOARD_VIEWBOX.height);
    }
  });

  it("gives every space a distinct position", () => {
    const positions = new Set(BOARD_SPACES.map(({ x, y }) => `${x},${y}`));
    expect(positions.size).toBe(BOARD_SPACES.length);
  });

  it("wraps from the finish line back to the start", () => {
    expect(START_SPACE).toBe(1);
    expect(FINISH_AFTER_SPACE).toBe(16);
  });
});
