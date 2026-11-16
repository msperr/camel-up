import { useState } from "react";
import {
  BOARD_SPACES,
  BOARD_VIEWBOX,
  CENTER_PANEL,
  DICE_ROW_Y,
  DICE_SIZE,
  DICE_TRAY,
  FINISH_LINE,
  getDiceRowCenters,
  getSpaceNumberPlacement,
} from "./boardLayout";

type DieValue = 1 | 2 | 3;

type DieDefinition = {
  readonly name: string;
  readonly label: string;
  readonly color: string;
  readonly edge: string;
  readonly rotation: number;
};

type Die = DieDefinition & {
  readonly value: DieValue;
};

const PIPS: Record<DieValue, readonly (readonly [number, number])[]> = {
  1: [[0, 0]],
  2: [
    [-15, -15],
    [15, 15],
  ],
  3: [
    [-15, -15],
    [0, 0],
    [15, 15],
  ],
};

const DIE_DEFINITIONS: readonly DieDefinition[] = [
  {
    name: "white",
    label: "White",
    color: "#fbf6ea",
    edge: "#7e6c55",
    rotation: -8,
  },
  {
    name: "yellow",
    label: "Yellow",
    color: "#f4c84a",
    edge: "#9a7217",
    rotation: 5,
  },
  {
    name: "orange",
    label: "Orange",
    color: "#ea8a3c",
    edge: "#a6531a",
    rotation: -3,
  },
  {
    name: "green",
    label: "Green",
    color: "#6aa55f",
    edge: "#3a6d34",
    rotation: 7,
  },
  {
    name: "blue",
    label: "Blue",
    color: "#5b8ec2",
    edge: "#2f5a86",
    rotation: -6,
  },
];

const INITIAL_VALUES: readonly DieValue[] = [1, 2, 3, 2, 1];

const createInitialDice = (): readonly Die[] =>
  DIE_DEFINITIONS.map((die, index) => ({
    ...die,
    value: INITIAL_VALUES[index] ?? 1,
  }));

const randomDieValue = (): DieValue => (Math.floor(Math.random() * 3) + 1) as DieValue;

function RollIcon() {
  return (
    <svg className="control-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <rect
        x="7"
        y="7.5"
        width="11"
        height="11"
        rx="3"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.8"
      />
      <circle cx="10.5" cy="11" r="1.15" fill="currentColor" />
      <circle cx="14.5" cy="15" r="1.15" fill="currentColor" />
      <path
        d="M5.5 8.5A7.5 7.5 0 0 1 17 4.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
      />
      <path
        d="M17.5 2.5v3.5h-3.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export default function App() {
  const [dice, setDice] = useState<readonly Die[]>(createInitialDice);
  const diceCenters = getDiceRowCenters(dice.length);
  const hasDice = dice.length > 0;

  const rollDice = () => {
    setDice(
      DIE_DEFINITIONS.map((die) => ({
        ...die,
        value: randomDieValue(),
      })),
    );
  };

  const clearDice = () => {
    setDice([]);
  };

  const diceStatus = hasDice
    ? `Dice: ${dice.map((die) => `${die.label} ${die.value}`).join(", ")}.`
    : "No dice";

  return (
    <main className="page">
      <h1 className="sr-only">Camel Up board</h1>

      <div className="game">
        <div className="board-frame">
          <svg
            className="board"
            viewBox={`0 0 ${BOARD_VIEWBOX.width} ${BOARD_VIEWBOX.height}`}
            role="img"
            aria-labelledby="board-title"
            aria-describedby="board-desc"
          >
            <title id="board-title">Camel Up board</title>
            <desc id="board-desc">
              Square Camel Up board with five rows and five columns. Sixteen dark track spaces run
              clockwise around the edge; space 1 is at the top middle, space 16 directly to its
              left, and the checkered finish line sits between them. Small numbers appear on the
              inner edges of the twelve non-corner spaces. The light central three by three panel
              contains a centred three by one tray for five coloured dice.
            </desc>

            <defs>
              <filter id="die-shadow" x="-50%" y="-50%" width="200%" height="220%">
                <feDropShadow
                  dx="0"
                  dy="4"
                  stdDeviation="3"
                  floodColor="#3a2d25"
                  floodOpacity="0.28"
                />
              </filter>
              <pattern id="finish-checker" width="10" height="10" patternUnits="userSpaceOnUse">
                <rect width="10" height="10" fill="#f8efd8" />
                <rect width="5" height="5" fill="#3a2d25" />
                <rect x="5" y="5" width="5" height="5" fill="#3a2d25" />
              </pattern>
            </defs>

            <rect width={BOARD_VIEWBOX.width} height={BOARD_VIEWBOX.height} fill="#e3b87a" />

            <g role="group" aria-label="Track spaces">
              {BOARD_SPACES.map((space) => {
                const number = getSpaceNumberPlacement(space);

                return (
                  <g
                    key={space.space}
                    role="group"
                    aria-label={`Space ${space.space}`}
                    data-space={space.space}
                    data-column={space.column}
                    data-row={space.row}
                  >
                    <rect
                      x={space.x - space.width / 2}
                      y={space.y - space.height / 2}
                      width={space.width}
                      height={space.height}
                      className="space-tile"
                      shapeRendering="crispEdges"
                    />
                    {number && (
                      <text
                        x={0}
                        y={0}
                        transform={`translate(${number.x} ${number.y}) rotate(${number.rotation})`}
                        textAnchor="middle"
                        dominantBaseline="central"
                        className="space-number"
                        data-number={space.space}
                        data-rotation={number.rotation}
                      >
                        {space.space}
                      </text>
                    )}
                  </g>
                );
              })}
            </g>

            <g aria-hidden="true" data-center-panel="true">
              <rect
                x={CENTER_PANEL.x}
                y={CENTER_PANEL.y}
                width={CENTER_PANEL.width}
                height={CENTER_PANEL.height}
                className="center-panel"
                shapeRendering="crispEdges"
              />

              <rect
                x={DICE_TRAY.x}
                y={DICE_TRAY.y}
                width={DICE_TRAY.width}
                height={DICE_TRAY.height}
                className="dice-tray"
                data-dice-tray="true"
                shapeRendering="crispEdges"
              />

              {dice.map((die, index) => (
                <g
                  key={die.name}
                  data-die={die.name}
                  data-value={die.value}
                  transform={`translate(${diceCenters[index]} ${DICE_ROW_Y}) rotate(${die.rotation})`}
                  filter="url(#die-shadow)"
                >
                  <rect
                    x={-DICE_SIZE / 2}
                    y={-DICE_SIZE / 2}
                    width={DICE_SIZE}
                    height={DICE_SIZE}
                    rx="14"
                    fill={die.color}
                    stroke={die.edge}
                    strokeWidth="3"
                  />
                  <rect
                    x={-DICE_SIZE / 2 + 7}
                    y={-DICE_SIZE / 2 + 7}
                    width={DICE_SIZE - 14}
                    height={DICE_SIZE - 14}
                    rx="9"
                    fill="none"
                    stroke="#ffffff"
                    strokeOpacity="0.3"
                    strokeWidth="2"
                  />
                  {PIPS[die.value].map(([pipX, pipY]) => (
                    <circle
                      key={`${pipX}-${pipY}`}
                      cx={pipX}
                      cy={pipY}
                      r={die.value === 1 ? 8.5 : 5.5}
                      fill="#33261d"
                    />
                  ))}
                </g>
              ))}
            </g>

            <g aria-hidden="true" data-finish-line="true">
              <rect
                x={FINISH_LINE.x - FINISH_LINE.width / 2}
                y={FINISH_LINE.y}
                width={FINISH_LINE.width}
                height={FINISH_LINE.height}
                fill="url(#finish-checker)"
                shapeRendering="crispEdges"
              />
            </g>

            <rect
              x="2"
              y="2"
              width={BOARD_VIEWBOX.width - 4}
              height={BOARD_VIEWBOX.height - 4}
              fill="none"
              stroke="#5b3520"
              strokeWidth="4"
              shapeRendering="crispEdges"
            />
          </svg>
        </div>

        <div className="dice-controls" data-dice-controls="true">
          <button
            type="button"
            className="dice-control-button roll-button"
            aria-label="Roll dice"
            title="Roll dice"
            onClick={rollDice}
          >
            <RollIcon />
          </button>
          <button
            type="button"
            className="dice-control-button clear-button"
            onClick={clearDice}
            disabled={!hasDice}
          >
            Clear
          </button>
        </div>
      </div>

      <p className="sr-only" role="status" aria-live="polite" data-dice-status="true">
        {diceStatus}
      </p>
    </main>
  );
}
