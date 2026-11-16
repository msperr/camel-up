import {
  BOARD_SPACES,
  BOARD_VIEWBOX,
  CENTER_PANEL,
  FINISH_LINE,
  getSpaceNumberPlacement,
} from "./boardLayout";

const DIE_SIZE = 56;

type DieValue = 1 | 2 | 3;

type Die = {
  readonly name: string;
  readonly color: string;
  readonly edge: string;
  readonly value: DieValue;
  readonly x: number;
  readonly y: number;
  readonly rotation: number;
};

const PIPS: Record<DieValue, readonly (readonly [number, number])[]> = {
  1: [[0, 0]],
  2: [
    [-12, -12],
    [12, 12],
  ],
  3: [
    [-12, -12],
    [0, 0],
    [12, 12],
  ],
};

const DICE: readonly Die[] = [
  {
    name: "white",
    color: "#fbf6ea",
    edge: "#7e6c55",
    value: 1,
    x: 350,
    y: 716,
    rotation: -8,
  },
  {
    name: "yellow",
    color: "#f4c84a",
    edge: "#9a7217",
    value: 2,
    x: 425,
    y: 704,
    rotation: 5,
  },
  {
    name: "orange",
    color: "#ea8a3c",
    edge: "#a6531a",
    value: 3,
    x: 500,
    y: 698,
    rotation: -3,
  },
  {
    name: "green",
    color: "#6aa55f",
    edge: "#3a6d34",
    value: 2,
    x: 575,
    y: 704,
    rotation: 7,
  },
  {
    name: "blue",
    color: "#5b8ec2",
    edge: "#2f5a86",
    value: 1,
    x: 650,
    y: 716,
    rotation: -6,
  },
];

export default function App() {
  return (
    <main className="page">
      <h1 className="sr-only">Camel Up Spielfeld</h1>

      <div className="board-frame">
        <svg
          className="board"
          viewBox={`0 0 ${BOARD_VIEWBOX.width} ${BOARD_VIEWBOX.height}`}
          role="img"
          aria-labelledby="board-title"
          aria-describedby="board-desc"
        >
          <title id="board-title">Camel Up Spielfeld</title>
          <desc id="board-desc">
            Quadratisches Spielfeld mit fünf mal fünf Feldern. Die 16 dunklen Rennfelder liegen
            fugenlos im Uhrzeigersinn am Rand; Feld 1 ist oben in der Mitte, Feld 16 direkt links
            daneben, dazwischen verläuft die Ziellinie. Die Zahlen stehen klein am jeweils inneren
            Rand; die vier Eckfelder sind leer. Im hellen Zentrum steht eine Pyramide mit fünf
            farbigen Würfeln.
          </desc>

          <defs>
            <linearGradient id="pyramid-gold" x1="0" y1="0" x2="0.2" y2="1">
              <stop offset="0%" stopColor="#f6cd7c" />
              <stop offset="55%" stopColor="#e5ac55" />
              <stop offset="100%" stopColor="#cf8f39" />
            </linearGradient>
            <radialGradient id="sun" cx="50%" cy="45%" r="55%">
              <stop offset="0%" stopColor="#f8bd4d" stopOpacity="0.85" />
              <stop offset="100%" stopColor="#f0a63c" stopOpacity="0.02" />
            </radialGradient>
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

          <g role="group" aria-label="Spielfelder">
            {BOARD_SPACES.map((space) => {
              const number = getSpaceNumberPlacement(space);

              return (
                <g
                  key={space.space}
                  role="group"
                  aria-label={`Feld ${space.space}`}
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

            <g transform="translate(0 10)">
              <circle cx="500" cy="468" r="152" fill="url(#sun)" />
              <text x="500" y="264" textAnchor="middle" className="board-title">
                CAMEL UP
              </text>
              <text x="500" y="292" textAnchor="middle" className="board-subtitle">
                DAS SPIELFELD
              </text>

              <ellipse cx="500" cy="652" rx="210" ry="16" fill="#5b3520" opacity="0.14" />

              <g data-pyramid="true">
                <path
                  d="M500 338 L312 648 L688 648 Z"
                  fill="url(#pyramid-gold)"
                  stroke="#8a5427"
                  strokeWidth="5"
                  strokeLinejoin="round"
                />
                <path d="M500 338 L688 648 L500 648 Z" fill="#bd7f36" opacity="0.55" />
                <path d="M500 338 L312 648 L500 648 Z" fill="#f7d38a" opacity="0.32" />
                <path d="M500 338 L500 648" stroke="#8a5427" strokeWidth="3" opacity="0.2" />
                <g stroke="#8a5427" strokeWidth="3" strokeLinecap="round" opacity="0.18">
                  <path d="M444 430 H556" />
                  <path d="M402 500 H598" />
                  <path d="M359 570 H641" />
                </g>
                <path
                  d="M455 648 L545 648 L524 588 L476 588 Z"
                  fill="#4a2f1c"
                  stroke="#8a5427"
                  strokeWidth="3"
                  strokeLinejoin="round"
                />
                <path d="M470 648 L530 648 L518 606 L482 606 Z" fill="#2f1c11" opacity="0.7" />
              </g>

              {DICE.map((die) => (
                <g
                  key={die.name}
                  data-die={die.name}
                  transform={`translate(${die.x} ${die.y}) rotate(${die.rotation})`}
                  filter="url(#die-shadow)"
                >
                  <rect
                    x={-DIE_SIZE / 2}
                    y={-DIE_SIZE / 2}
                    width={DIE_SIZE}
                    height={DIE_SIZE}
                    rx="11"
                    fill={die.color}
                    stroke={die.edge}
                    strokeWidth="3"
                  />
                  <rect
                    x={-DIE_SIZE / 2 + 5}
                    y={-DIE_SIZE / 2 + 5}
                    width={DIE_SIZE - 10}
                    height={DIE_SIZE - 10}
                    rx="7"
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
                      r={die.value === 1 ? 7 : 4.6}
                      fill="#33261d"
                    />
                  ))}
                </g>
              ))}
            </g>
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
    </main>
  );
}
