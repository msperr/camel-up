import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
} from "react";
import {
  BOARD_SPACES,
  BOARD_VIEWBOX,
  CENTER_PANEL,
  DICE_ADD_BUTTON,
  DICE_ROW_Y,
  DICE_SIZE,
  DICE_TRAY,
  FINISH_LINE,
  getDiceRowCenters,
  getSpaceNumberPlacement,
} from "./boardLayout";

type DieValue = 1 | 2 | 3;

type CamelColor = "white" | "yellow" | "orange" | "green" | "blue";

type DieStyle = {
  readonly label: string;
  readonly color: string;
  readonly edge: string;
};

const CAMEL_COLORS: readonly CamelColor[] = ["white", "yellow", "orange", "green", "blue"];

const CAMEL_DIE_STYLES: Record<CamelColor, DieStyle> = {
  white: { label: "White", color: "#fbf6ea", edge: "#7e6c55" },
  yellow: { label: "Yellow", color: "#f4c84a", edge: "#9a7217" },
  orange: { label: "Orange", color: "#ea8a3c", edge: "#a6531a" },
  green: { label: "Green", color: "#6aa55f", edge: "#3a6d34" },
  blue: { label: "Blue", color: "#5b8ec2", edge: "#2f5a86" },
};

type ThrownDie = {
  readonly color: CamelColor;
  readonly value: DieValue;
};

const MAX_DICE = CAMEL_COLORS.length;

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

const SLOT_CENTERS = getDiceRowCenters(MAX_DICE);
const SLOT_ROTATIONS = [-8, 5, -3, 7, -6] as const;

const FOCUSABLE_SELECTOR =
  'button:not([disabled]), select:not([disabled]), input:not([disabled]), textarea:not([disabled]), a[href], [tabindex]:not([tabindex="-1"])';

const getFocusableElements = (container: HTMLElement): HTMLElement[] => [
  ...container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR),
];

type DieFaceProps = {
  readonly color: CamelColor;
  readonly value: DieValue;
  readonly x: number;
  readonly y: number;
  readonly rotation: number;
};

function DieFace({ color, value, x, y, rotation }: DieFaceProps) {
  const style = CAMEL_DIE_STYLES[color];

  return (
    <g
      data-die={color}
      data-value={value}
      transform={`translate(${x} ${y}) rotate(${rotation})`}
      filter="url(#die-shadow)"
    >
      <rect
        x={-DICE_SIZE / 2}
        y={-DICE_SIZE / 2}
        width={DICE_SIZE}
        height={DICE_SIZE}
        rx="14"
        fill={style.color}
        stroke={style.edge}
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
      {PIPS[value].map(([pipX, pipY]) => (
        <circle
          key={`${pipX}-${pipY}`}
          cx={pipX}
          cy={pipY}
          r={value === 1 ? 8.5 : 5.5}
          fill="#33261d"
        />
      ))}
    </g>
  );
}

function RandomIcon() {
  return (
    <svg className="dialog-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
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
  const [dice, setDice] = useState<readonly ThrownDie[]>([]);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedColor, setSelectedColor] = useState<CamelColor | "">("");
  const [selectedValue, setSelectedValue] = useState<DieValue | "">("");
  const [focusAfterClose, setFocusAfterClose] = useState<"add" | "clear" | null>(null);
  const addButtonRef = useRef<HTMLButtonElement>(null);
  const clearButtonRef = useRef<HTMLButtonElement>(null);
  const colorSelectRef = useRef<HTMLSelectElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);

  const remainingColors = CAMEL_COLORS.filter((color) => !dice.some((die) => die.color === color));
  const canAddDie = remainingColors.length > 0;
  const hasDice = dice.length > 0;

  const closeDialog = (focusTarget: "add" | "clear" = canAddDie ? "add" : "clear") => {
    setFocusAfterClose(focusTarget);
    setIsDialogOpen(false);
  };

  const openDialog = () => {
    if (!canAddDie) {
      return;
    }

    setSelectedColor("");
    setSelectedValue("");
    setIsDialogOpen(true);
  };

  useEffect(() => {
    if (!isDialogOpen) {
      return;
    }

    colorSelectRef.current?.focus();

    const handleKeyDown = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") {
        setFocusAfterClose(canAddDie ? "add" : "clear");
        setIsDialogOpen(false);
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [canAddDie, isDialogOpen]);

  useEffect(() => {
    if (!isDialogOpen) {
      return;
    }

    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";

    return () => {
      document.body.style.overflow = previousOverflow;
    };
  }, [isDialogOpen]);

  const handleDialogKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key !== "Tab") {
      return;
    }

    const dialog = dialogRef.current;

    if (!dialog) {
      return;
    }

    const focusable = getFocusableElements(dialog);

    if (focusable.length === 0) {
      event.preventDefault();
      dialog.focus();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;

    if (!(active instanceof Node) || !dialog.contains(active)) {
      event.preventDefault();
      first.focus();
      return;
    }

    if (event.shiftKey && (active === first || active === dialog)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  };

  useEffect(() => {
    if (isDialogOpen || focusAfterClose === null) {
      return;
    }

    const target = focusAfterClose === "add" ? addButtonRef.current : clearButtonRef.current;

    target?.focus();
    setFocusAfterClose(null);
  }, [focusAfterClose, isDialogOpen]);

  const randomizeDie = () => {
    if (remainingColors.length === 0) {
      return;
    }

    const color = remainingColors[Math.floor(Math.random() * remainingColors.length)];
    const value = (Math.floor(Math.random() * 3) + 1) as DieValue;

    setSelectedColor(color);
    setSelectedValue(value);
  };

  const confirmDie = () => {
    if (selectedColor === "" || selectedValue === "") {
      return;
    }

    if (dice.length >= MAX_DICE || dice.some((die) => die.color === selectedColor)) {
      return;
    }

    const willFillTray = dice.length + 1 >= MAX_DICE;

    setDice((currentDice) => [...currentDice, { color: selectedColor, value: selectedValue }]);
    closeDialog(willFillTray ? "clear" : "add");
  };

  const clearDice = () => {
    setDice([]);
    setFocusAfterClose("add");
  };

  const diceStatus = hasDice
    ? `Dice: ${dice.map((die) => `${CAMEL_DIE_STYLES[die.color].label} ${die.value}`).join(", ")}.`
    : "No dice";

  const addButtonStyle = {
    left: `${(DICE_ADD_BUTTON.x / BOARD_VIEWBOX.width) * 100}%`,
    top: `${(DICE_ADD_BUTTON.y / BOARD_VIEWBOX.height) * 100}%`,
    "--add-die-size": `${(DICE_ADD_BUTTON.size / BOARD_VIEWBOX.width) * 100}cqw`,
  } as CSSProperties;

  return (
    <main className="page">
      <div className="page-content" inert={isDialogOpen} data-page-content="true">
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
                contains a centred three by one tray for five coloured dice and an add-die button.
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

                {Array.from({ length: MAX_DICE }, (_, index) => {
                  const die = dice[index];
                  const centerX = SLOT_CENTERS[index];

                  return (
                    <g key={index} data-die-slot={index + 1}>
                      {!die && (
                        <rect
                          x={centerX - DICE_SIZE / 2}
                          y={DICE_ROW_Y - DICE_SIZE / 2}
                          width={DICE_SIZE}
                          height={DICE_SIZE}
                          rx="14"
                          className="die-slot"
                          shapeRendering="crispEdges"
                        />
                      )}
                      {die && (
                        <DieFace
                          color={die.color}
                          value={die.value}
                          x={centerX}
                          y={DICE_ROW_Y}
                          rotation={SLOT_ROTATIONS[index]}
                        />
                      )}
                    </g>
                  );
                })}
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

            <button
              ref={addButtonRef}
              type="button"
              className="add-die-button"
              aria-label="Add die"
              title="Add die"
              data-add-die="true"
              style={addButtonStyle}
              onClick={openDialog}
              disabled={!canAddDie}
            >
              <span aria-hidden="true">+</span>
            </button>
          </div>

          <div className="dice-controls" data-dice-controls="true">
            <button
              ref={clearButtonRef}
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
      </div>

      {isDialogOpen && (
        <div
          className="dice-dialog-backdrop"
          onClick={(event) => {
            if (event.button === 0 && event.target === event.currentTarget) {
              closeDialog();
            }
          }}
          onKeyDown={handleDialogKeyDown}
        >
          <div
            ref={dialogRef}
            className="dice-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="dice-dialog-title"
            tabIndex={-1}
          >
            <h2 id="dice-dialog-title">Add die</h2>

            <div className="dice-dialog-field">
              <label htmlFor="dice-color">Colour</label>
              <select
                id="dice-color"
                ref={colorSelectRef}
                value={selectedColor}
                onChange={(event) => {
                  const color = CAMEL_COLORS.find((item) => item === event.target.value);
                  setSelectedColor(color ?? "");
                }}
              >
                <option value="">Choose a colour</option>
                {remainingColors.map((color) => (
                  <option key={color} value={color}>
                    {CAMEL_DIE_STYLES[color].label}
                  </option>
                ))}
              </select>
            </div>

            <div className="dice-dialog-field">
              <label htmlFor="dice-value">Value</label>
              <select
                id="dice-value"
                value={selectedValue}
                onChange={(event) => {
                  const value = Number(event.target.value);
                  setSelectedValue(value === 1 || value === 2 || value === 3 ? value : "");
                }}
              >
                <option value="">Choose a value</option>
                <option value="1">1</option>
                <option value="2">2</option>
                <option value="3">3</option>
              </select>
            </div>

            <div className="dice-dialog-actions">
              <button
                type="button"
                className="dialog-button dialog-button--secondary dialog-button--icon"
                aria-label="Randomize selection"
                title="Randomize selection"
                onClick={randomizeDie}
                disabled={remainingColors.length === 0}
              >
                <RandomIcon />
              </button>
              <button
                type="button"
                className="dialog-button dialog-button--secondary"
                onClick={() => closeDialog()}
              >
                Cancel
              </button>
              <button
                type="button"
                className="dialog-button dialog-button--primary"
                onClick={confirmDie}
                disabled={selectedColor === "" || selectedValue === ""}
              >
                OK
              </button>
            </div>
          </div>
        </div>
      )}
    </main>
  );
}
