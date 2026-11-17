export const BOARD_VIEWBOX = { width: 1000, height: 1000 } as const;

/** The board is a square 5 x 5 grid with a 3 x 3 centre panel. */
export const GRID_SIZE = 5;
export const CELL_SIZE = BOARD_VIEWBOX.width / GRID_SIZE;
export const SPACE_WIDTH = CELL_SIZE;
export const SPACE_HEIGHT = CELL_SIZE;

/** The 2014 game has 16 numbered race-track spaces. */
export const START_SPACE = 1;
export const FINISH_AFTER_SPACE = 16;

/**
 * The finish line sits on the shared edge between fields 16 and 1. `x` is the
 * centre of the vertical checkered strip.
 */
export const FINISH_LINE = {
  x: CELL_SIZE * 2,
  y: 0,
  width: 10,
  height: CELL_SIZE,
} as const;

/** The central 3 x 3 cells, touching the inner edges of the outer fields. */
export const CENTER_PANEL = {
  x: CELL_SIZE,
  y: CELL_SIZE,
  width: CELL_SIZE * 3,
  height: CELL_SIZE * 3,
} as const;

/**
 * The 3 x 1 dice band. It is vertically centred in the central panel, leaving
 * an equal 200px gap above and below it.
 */
export const DICE_TRAY = {
  x: CENTER_PANEL.x,
  y: CENTER_PANEL.y + CENTER_PANEL.height / 3,
  width: CENTER_PANEL.width,
  height: CENTER_PANEL.height / 3,
} as const;

/** Five 72px dice with a 26px gap fit inside the 600px-wide tray. */
export const DICE_SIZE = 72;
export const DICE_GAP = 26;

/** The horizontal centre line of the dice row. */
export const DICE_ROW_Y = DICE_TRAY.y + DICE_TRAY.height / 2;

/**
 * The add-die button sits in the tray's lower-right corner. Its size is the
 * rendered diameter in viewBox units at the board's maximum width; CSS scales it
 * down on small boards and keeps a larger touch target around it.
 */
export const DICE_ADD_BUTTON = {
  x: 766,
  y: 568,
  size: 30,
} as const;

/**
 * Centre a row of dice in the tray. The first and last dice keep the same
 * distance from the tray's left and right edges.
 */
export const getDiceRowCenters = (count: number): readonly number[] => {
  if (count <= 0) {
    return [];
  }

  const rowWidth = count * DICE_SIZE + (count - 1) * DICE_GAP;
  const firstCenter = DICE_TRAY.x + (DICE_TRAY.width - rowWidth) / 2 + DICE_SIZE / 2;

  return Array.from({ length: count }, (_, index) => firstCenter + index * (DICE_SIZE + DICE_GAP));
};

/** Distance from a field's inner edge to the centre of its small number. */
export const NUMBER_EDGE_OFFSET = 20;

export type BoardSpace = {
  readonly space: number;
  readonly column: number;
  readonly row: number;
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
};

export type SpaceNumberPlacement = {
  readonly x: number;
  readonly y: number;
  readonly rotation: number;
};

const createSpace = (space: number, column: number, row: number): BoardSpace => ({
  space,
  column,
  row,
  x: (column - 0.5) * CELL_SIZE,
  y: (row - 0.5) * CELL_SIZE,
  width: SPACE_WIDTH,
  height: SPACE_HEIGHT,
});

/**
 * Static space-to-coordinate map for the SVG board.
 *
 * Coordinates are the centre of each space in the 1000 x 1000 viewBox.
 * The order is 1..16 clockwise around the outer ring; field 1 is the top
 * middle, field 16 is directly to its left, and the finish line runs between
 * them. Board presentation intentionally lives here, not in the Rust API.
 */
export const BOARD_SPACES: readonly BoardSpace[] = [
  createSpace(1, 3, 1),
  createSpace(2, 4, 1),
  createSpace(3, 5, 1),
  createSpace(4, 5, 2),
  createSpace(5, 5, 3),
  createSpace(6, 5, 4),
  createSpace(7, 5, 5),
  createSpace(8, 4, 5),
  createSpace(9, 3, 5),
  createSpace(10, 2, 5),
  createSpace(11, 1, 5),
  createSpace(12, 1, 4),
  createSpace(13, 1, 3),
  createSpace(14, 1, 2),
  createSpace(15, 1, 1),
  createSpace(16, 2, 1),
];

export const isCornerSpace = (space: BoardSpace): boolean =>
  (space.column === 1 || space.column === GRID_SIZE) &&
  (space.row === 1 || space.row === GRID_SIZE);

/**
 * Place a small number just inside the edge that faces the central panel.
 * The top of each digit points away from the centre of the board.
 */
export const getSpaceNumberPlacement = (space: BoardSpace): SpaceNumberPlacement | null => {
  if (isCornerSpace(space)) {
    return null;
  }

  if (space.row === 1) {
    return {
      x: space.x,
      y: space.y + space.height / 2 - NUMBER_EDGE_OFFSET,
      rotation: 0,
    };
  }

  if (space.column === GRID_SIZE) {
    return {
      x: space.x - space.width / 2 + NUMBER_EDGE_OFFSET,
      y: space.y,
      rotation: 90,
    };
  }

  if (space.row === GRID_SIZE) {
    return {
      x: space.x,
      y: space.y - space.height / 2 + NUMBER_EDGE_OFFSET,
      rotation: 180,
    };
  }

  return {
    x: space.x + space.width / 2 - NUMBER_EDGE_OFFSET,
    y: space.y,
    rotation: -90,
  };
};
