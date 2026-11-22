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
  LEG_BET_BAND,
  LEG_BET_CARD_SIZE,
  LEG_BET_STACK_COUNT,
  getDiceRowCenters,
  getFinishOverflowCenters,
  getLegBetCardPlacements,
  getLegBetStackCenters,
  getSpaceCamelCenters,
  getSpaceNumberPlacement,
} from "./boardLayout";
import {
  CAMEL_COLORS,
  applyDieWithDeserts,
  createRaceState,
  hasCamels,
  isRaceFinished,
  type CamelColor,
  type DieValue,
  type RaceState,
} from "./camelRace";
import {
  createPlacedDesert,
  DESERT_ART_VIEWBOX,
  DESERT_PLUS_ARM,
  DESERT_PLUS_INSET,
  getDesertArtTransform,
  getDesertTilePlacement,
  isLegalDesertPlacement,
  type DesertKind,
  type DesertTilePlacement,
  type PlacedDesert,
} from "./desertTiles";
import {
  createLegBetState,
  getLegBetValues,
  hasTakenLegBet,
  takeLegBet,
  type LegBetState,
  type LegBetValue,
} from "./legBets";
import DesertArtwork from "./DesertArtwork";
import Cockpit from "./Cockpit";
import {
  buildCockpitRequestWithDeserts,
  fetchCockpitEvaluation,
  type CockpitResponse,
} from "./cockpit";

type DieStyle = {
  readonly label: string;
  readonly color: string;
  readonly edge: string;
};

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

/** One dialog at a time; the desert dialog remembers the field it came from. */
type DialogState =
  { readonly kind: "dice" } | { readonly kind: "desert"; readonly space: number } | null;

type FocusTarget = "add" | "clear" | "next" | { readonly space: number };

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
const LEG_BET_CENTERS = getLegBetStackCenters(LEG_BET_STACK_COUNT);

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

type CamelPieceProps = {
  readonly color: CamelColor;
  readonly x: number;
  readonly y: number;
  readonly space?: number;
  readonly stackIndex: number;
  readonly finished?: boolean;
};

type CamelSilhouetteProps = {
  readonly color: string;
  readonly edge: string;
};

/** The side-view camel artwork, shared by board pieces and leg-bet cards. */
function CamelSilhouette({ color, edge }: CamelSilhouetteProps) {
  return (
    <g fill={color} stroke={edge} strokeWidth="1.6" strokeLinejoin="round">
      <path
        d="M -27 1 q -7 2 -9 9"
        fill="none"
        stroke={edge}
        strokeWidth="2.6"
        strokeLinecap="round"
      />
      <rect x={-26} y={5} width={6} height={13} rx={2} />
      <rect x={-13} y={5} width={6} height={13} rx={2} />
      <rect x={4} y={5} width={6} height={13} rx={2} />
      <rect x={16} y={5} width={6} height={13} rx={2} />
      <ellipse cx={-2} cy={3} rx={25} ry={9} />
      <path d="M -22 -3 C -18 -18 -4 -18 2 -4 Z" />
      <path d="M 12 2 C 17 -3 18 -13 24 -15 L 30 -15 L 30 -10 L 26 -9 C 24 -8 23 -3 22 3 Z" />
      <ellipse cx={29} cy={-13} rx={6} ry={4.5} />
      <path d="M 26 -16 l 2 -3 2 3 Z" />
      <circle cx={31} cy={-14} r={1.1} fill="#33261d" stroke="none" />
    </g>
  );
}

/**
 * A side-view camel meeple, drawn around its centre at (x, y). The silhouette
 * stays inside the camel's fixed footprint so five fit in one board field.
 */
function CamelPiece({ color, x, y, space, stackIndex, finished = false }: CamelPieceProps) {
  const style = CAMEL_DIE_STYLES[color];

  return (
    <g
      data-camel={color}
      data-space={space}
      data-stack-index={stackIndex}
      data-finished={finished ? "true" : undefined}
      transform={`translate(${x} ${y})`}
      filter="url(#camel-shadow)"
    >
      <CamelSilhouette color={style.color} edge={style.edge} />
    </g>
  );
}

/** A flat cardboard tile drawn inside its track field, behind the camel pieces. */
function DesertTile({
  desert,
  placement,
}: {
  readonly desert: PlacedDesert;
  readonly placement: DesertTilePlacement;
}) {
  return (
    <g
      data-desert={desert.kind}
      data-desert-space={desert.space}
      transform={getDesertArtTransform(placement)}
      filter="url(#tile-shadow)"
    >
      <rect
        width={DESERT_ART_VIEWBOX}
        height={DESERT_ART_VIEWBOX}
        rx="10"
        className="desert-tile-surface"
      />
      <rect x="6" y="6" width="88" height="88" rx="7" className="desert-tile-inner" />
      <DesertArtwork kind={desert.kind} />
    </g>
  );
}

type LegBetCardProps = {
  readonly color: CamelColor;
  readonly value: LegBetValue;
  readonly x: number;
  readonly y: number;
  /** Only the fully visible top tile can be taken. */
  readonly topCard: boolean;
  readonly enabled: boolean;
  readonly onTake: (color: CamelColor) => void;
};

/** One square tile in a camel's 5/3/2 leg-bet stack. */
function LegBetCard({ color, value, x, y, topCard, enabled, onTake }: LegBetCardProps) {
  const style = CAMEL_DIE_STYLES[color];
  const label = `Take ${style.label} ${value} leg-bet tile`;
  const handleClick = () => {
    if (topCard && enabled) {
      onTake(color);
    }
  };

  return (
    <g
      data-leg-bet-card={color}
      data-leg-bet-value={value}
      data-leg-bet-available={topCard && enabled ? "true" : undefined}
      className={
        topCard
          ? `leg-bet-card${enabled ? " leg-bet-card--available" : ""}`
          : "leg-bet-card leg-bet-card--covered"
      }
      role={topCard ? "button" : undefined}
      tabIndex={topCard ? (enabled ? 0 : -1) : undefined}
      aria-label={topCard ? label : undefined}
      aria-disabled={topCard && !enabled ? "true" : undefined}
      aria-hidden={topCard ? undefined : true}
      transform={`translate(${x} ${y})`}
      filter="url(#card-shadow)"
      onClick={handleClick}
      onKeyDown={(event) => {
        if (topCard && enabled && (event.key === "Enter" || event.key === " ")) {
          event.preventDefault();
          handleClick();
        }
      }}
    >
      <rect
        width={LEG_BET_CARD_SIZE}
        height={LEG_BET_CARD_SIZE}
        rx="10"
        fill={style.color}
        stroke={style.edge}
        strokeWidth="3"
      />
      <rect
        x="7"
        y="7"
        width={LEG_BET_CARD_SIZE - 14}
        height={LEG_BET_CARD_SIZE - 14}
        rx="7"
        fill="none"
        stroke="#ffffff"
        strokeOpacity="0.4"
        strokeWidth="2"
      />
      <g transform={`translate(${LEG_BET_CARD_SIZE / 2} 44) scale(0.62)`}>
        <CamelSilhouette color={style.color} edge={style.edge} />
      </g>
      <text
        x={LEG_BET_CARD_SIZE / 2}
        y={LEG_BET_CARD_SIZE - 14}
        textAnchor="middle"
        className="leg-bet-value"
        fill={style.edge}
      >
        {value}
      </text>
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
  const [race, setRace] = useState<RaceState>(createRaceState);
  const [legBets, setLegBets] = useState<LegBetState>(createLegBetState);
  const [deserts, setDeserts] = useState<readonly PlacedDesert[]>([]);
  const [isSetupRound, setIsSetupRound] = useState(true);
  const [cockpit, setCockpit] = useState<CockpitResponse | null>(null);
  const [cockpitError, setCockpitError] = useState<string | null>(null);
  const [dialog, setDialog] = useState<DialogState>(null);
  const [selectedColor, setSelectedColor] = useState<CamelColor | "">("");
  const [selectedValue, setSelectedValue] = useState<DieValue | "">("");
  const [focusAfterClose, setFocusAfterClose] = useState<FocusTarget | null>(null);
  const addButtonRef = useRef<HTMLButtonElement>(null);
  const clearButtonRef = useRef<HTMLButtonElement>(null);
  const nextRoundButtonRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);

  const remainingColors = CAMEL_COLORS.filter((color) => !dice.some((die) => die.color === color));
  const raceFinished = isRaceFinished(race);
  const camelsOnBoard = hasCamels(race);
  const hasDice = dice.length > 0;
  const canAddDie = remainingColors.length > 0 && !raceFinished;
  const canStartNextRound = dice.length === MAX_DICE && !raceFinished;
  // Setup dice only place the camels; leg betting and desert tiles start with the first leg.
  const canTakeLegBet = !isSetupRound && dice.length < MAX_DICE && !raceFinished;

  const legalDesertSpaces = BOARD_SPACES.filter((space) =>
    isLegalDesertPlacement(space.space, race, deserts),
  ).map((space) => space.space);
  const canPlaceDesert = canTakeLegBet && camelsOnBoard && legalDesertSpaces.length > 0;

  const canClear = hasDice || camelsOnBoard || hasTakenLegBet(legBets) || deserts.length > 0;

  const isDialogOpen = dialog !== null;

  const defaultFocusTarget = (): FocusTarget => {
    if (dialog?.kind === "desert") {
      return { space: dialog.space };
    }
    return canAddDie ? "add" : canStartNextRound ? "next" : "clear";
  };

  const closeDialog = (focusTarget: FocusTarget = defaultFocusTarget()) => {
    setFocusAfterClose(focusTarget);
    setDialog(null);
  };

  const openDiceDialog = () => {
    if (!canAddDie) {
      return;
    }

    setSelectedColor("");
    setSelectedValue("");
    setDialog({ kind: "dice" });
  };

  const openDesertDialog = (space: number) => {
    if (!canPlaceDesert || !legalDesertSpaces.includes(space)) {
      return;
    }

    setDialog({ kind: "desert", space });
  };

  const placeDesert = (kind: DesertKind) => {
    if (dialog?.kind !== "desert") {
      return;
    }

    const space = dialog.space;
    if (!legalDesertSpaces.includes(space)) {
      closeDialog();
      return;
    }

    const placed = [...deserts, createPlacedDesert(space, kind)];
    const nextSpace = [
      ...legalDesertSpaces.filter((candidate) => candidate > space),
      ...legalDesertSpaces.filter((candidate) => candidate < space),
    ].find((candidate) => isLegalDesertPlacement(candidate, race, placed));

    setDeserts(placed);
    closeDialog(nextSpace === undefined ? "add" : { space: nextSpace });
  };

  useEffect(() => {
    if (!isDialogOpen) {
      return;
    }

    dialogRef.current?.querySelector<HTMLElement>(FOCUSABLE_SELECTOR)?.focus();

    const handleKeyDown = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") {
        setFocusAfterClose(
          dialog?.kind === "desert"
            ? { space: dialog.space }
            : canAddDie
              ? "add"
              : canStartNextRound
                ? "next"
                : "clear",
        );
        setDialog(null);
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [canAddDie, canStartNextRound, dialog, isDialogOpen]);

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

    const target =
      typeof focusAfterClose === "string"
        ? focusAfterClose === "add"
          ? addButtonRef.current
          : focusAfterClose === "next"
            ? nextRoundButtonRef.current
            : clearButtonRef.current
        : (document.querySelector<HTMLElement>(
            `[data-desert-place-space="${focusAfterClose.space}"]`,
          ) ??
          document.querySelector<HTMLElement>("[data-desert-place-space]") ??
          addButtonRef.current);

    target?.focus();
    setFocusAfterClose(null);
  }, [focusAfterClose, isDialogOpen]);

  useEffect(() => {
    const controller = new AbortController();

    setCockpitError(null);
    setCockpit(null);

    fetchCockpitEvaluation(
      buildCockpitRequestWithDeserts(dice, race, legBets, isSetupRound, deserts),
      controller.signal,
    )
      .then((evaluation) => {
        setCockpit(evaluation);
        setCockpitError(null);
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) {
          return;
        }
        console.error("Could not evaluate cockpit actions", error);
        setCockpit(null);
        setCockpitError(error instanceof Error ? error.message : String(error));
      });

    return () => controller.abort();
  }, [dice, race, legBets, isSetupRound, deserts]);

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
    const nextRace = applyDieWithDeserts(race, selectedColor, selectedValue, deserts);
    const focusTarget: FocusTarget = isRaceFinished(nextRace)
      ? "clear"
      : willFillTray
        ? "next"
        : "add";

    setDice((currentDice) => [...currentDice, { color: selectedColor, value: selectedValue }]);
    setRace(nextRace);
    closeDialog(focusTarget);
  };

  const handleTakeLegBet = (color: CamelColor) => {
    if (!canTakeLegBet) {
      return;
    }

    if (legBets[color] === 1) {
      setFocusAfterClose("clear");
    }

    setLegBets((current) => takeLegBet(current, color));
  };

  const clearBoard = () => {
    setDice([]);
    setRace(createRaceState());
    setLegBets(createLegBetState());
    setDeserts([]);
    setIsSetupRound(true);
    setFocusAfterClose("add");
  };

  const startNextRound = () => {
    if (!canStartNextRound) {
      return;
    }

    setDice([]);
    setLegBets(createLegBetState());
    setDeserts([]);
    setIsSetupRound(false);
    setFocusAfterClose("add");
  };

  const diceStatus = hasDice
    ? `Dice: ${dice.map((die) => `${CAMEL_DIE_STYLES[die.color].label} ${die.value}`).join(", ")}.`
    : "No dice";

  const camelPositions = BOARD_SPACES.flatMap((space) =>
    (race.stacks[space.space] ?? []).map(
      (color) => `${CAMEL_DIE_STYLES[color].label} on ${space.space}`,
    ),
  );

  const camelStatus = raceFinished
    ? `Race finished: ${race.finished?.map((color) => CAMEL_DIE_STYLES[color].label).join(", ")} past the finish line.`
    : camelPositions.length > 0
      ? `Camels: ${camelPositions.join(", ")}.`
      : "No camels";

  const desertStatus =
    deserts.length > 0
      ? `Deserts: ${deserts.map((desert) => `${desert.kind} on ${desert.space}`).join(", ")}.`
      : "No deserts";

  const addButtonStyle = {
    left: `${(DICE_ADD_BUTTON.x / BOARD_VIEWBOX.width) * 100}%`,
    top: `${(DICE_ADD_BUTTON.y / BOARD_VIEWBOX.height) * 100}%`,
    "--add-die-size": `${(DICE_ADD_BUTTON.size / BOARD_VIEWBOX.width) * 100}cqw`,
  } as CSSProperties;

  // Keep the CSS band sizing in sync with the SVG layout constants.
  const bettingBandStyle = {
    "--bet-scale": 1 + LEG_BET_BAND.height / BOARD_VIEWBOX.width,
    "--bet-band-aspect": `${BOARD_VIEWBOX.width} / ${LEG_BET_BAND.height}`,
  } as CSSProperties;

  const camelPieces = BOARD_SPACES.flatMap((space) => {
    const stack = race.stacks[space.space] ?? [];
    const centers = getSpaceCamelCenters(space.space, stack.length);

    return stack.map((color, index) => (
      <CamelPiece
        key={`${space.space}-${index}-${color}`}
        color={color}
        x={centers[index].x}
        y={centers[index].y}
        space={space.space}
        stackIndex={index}
      />
    ));
  });

  const finishedCenters = race.finished ? getFinishOverflowCenters(race.finished.length) : [];

  const desertPieces = deserts.flatMap((desert) => {
    const placement = getDesertTilePlacement(desert.space);
    return placement === null
      ? []
      : [<DesertTile key={desert.space} desert={desert} placement={placement} />];
  });

  const legBetStacks = CAMEL_COLORS.map((color, colorIndex) => {
    const remaining = legBets[color];
    const values = getLegBetValues(remaining);
    const centerX = LEG_BET_CENTERS[colorIndex];
    const placements = getLegBetCardPlacements(centerX, values.length);

    if (placements.length === 0) {
      const empty = getLegBetCardPlacements(centerX, 1)[0];

      return (
        <rect
          key={`empty-${color}`}
          data-leg-bet-empty={color}
          className="leg-bet-empty"
          x={empty.x}
          y={empty.y}
          width={LEG_BET_CARD_SIZE}
          height={LEG_BET_CARD_SIZE}
          rx="10"
          aria-hidden="true"
        />
      );
    }

    // Paint covered tiles first so the fully visible top tile is on top. Keying
    // by stack position keeps the focused node when the tile it shows changes.
    return values
      .map((value, index) => ({
        key: `${color}-${index}`,
        value,
        placement: placements[index],
        topCard: index === 0,
      }))
      .reverse()
      .map(({ key, value, placement, topCard }) => (
        <LegBetCard
          key={key}
          color={color}
          value={value}
          x={placement.x}
          y={placement.y}
          topCard={topCard}
          enabled={canTakeLegBet}
          onTake={handleTakeLegBet}
        />
      ));
  });

  return (
    <main className="page">
      <div className="page-content" inert={isDialogOpen} data-page-content="true">
        <h1 className="sr-only">Camel Up board</h1>

        <div className="game" style={bettingBandStyle}>
          <div className="board-stack">
            <svg
              className="betting-field"
              viewBox={`${LEG_BET_BAND.x} ${LEG_BET_BAND.y} ${LEG_BET_BAND.width} ${LEG_BET_BAND.height}`}
              role="group"
              aria-labelledby="betting-title"
              aria-describedby="betting-desc"
            >
              <title id="betting-title">Leg-betting tiles</title>
              <desc id="betting-desc">
                Five stacks of square tiles, one per camel, printed 5, 3, and 2. Only the fully
                visible top tile of each stack can be taken.
              </desc>
              <defs>
                <filter id="card-shadow" x="-30%" y="-30%" width="160%" height="170%">
                  <feDropShadow
                    dx="0"
                    dy="3"
                    stdDeviation="2.5"
                    floodColor="#3a2d25"
                    floodOpacity="0.3"
                  />
                </filter>
              </defs>
              <g data-leg-bets="true">
                <rect
                  x={LEG_BET_BAND.x}
                  y={LEG_BET_BAND.y}
                  width={LEG_BET_BAND.width}
                  height={LEG_BET_BAND.height}
                  className="leg-bet-band"
                  aria-hidden="true"
                />
                <rect
                  x={LEG_BET_BAND.x + 2}
                  y={LEG_BET_BAND.y + 2}
                  width={LEG_BET_BAND.width - 4}
                  height={LEG_BET_BAND.height - 4}
                  fill="none"
                  stroke="#5b3520"
                  strokeWidth="4"
                  shapeRendering="crispEdges"
                  aria-hidden="true"
                />
                {legBetStacks}
              </g>
            </svg>

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
                  Square Camel Up board with five rows and five columns. Sixteen dark track spaces
                  run clockwise around the edge; space 1 is at the top middle, space 16 directly to
                  its left, and the checkered finish line sits between them. Small numbers appear on
                  the inner edges of the twelve non-corner spaces. Coloured camel pieces stand on
                  the fields, stacked tightly when they share one. A camel that passes the finish
                  line is drawn just above it, outside the track. The light central three by three
                  panel contains a centred three by one tray for five coloured dice and an add-die
                  button. Flat desert tiles may lie on empty fields, underneath any passing camels.
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
                  <filter id="camel-shadow" x="-30%" y="-30%" width="160%" height="180%">
                    <feDropShadow
                      dx="0"
                      dy="2"
                      stdDeviation="1.5"
                      floodColor="#3a2d25"
                      floodOpacity="0.25"
                    />
                  </filter>
                  <filter id="tile-shadow" x="-30%" y="-30%" width="160%" height="180%">
                    <feDropShadow
                      dx="0"
                      dy="2"
                      stdDeviation="2"
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

                {desertPieces.length > 0 && (
                  <g aria-hidden="true" data-deserts="true">
                    {desertPieces}
                  </g>
                )}

                {camelPieces.length > 0 && (
                  <g aria-hidden="true" data-camels="true">
                    {camelPieces}
                  </g>
                )}

                {race.finished && (
                  <g aria-hidden="true" data-finished-camels="true">
                    {race.finished.map((color, index) => (
                      <CamelPiece
                        key={`finished-${index}-${color}`}
                        color={color}
                        x={finishedCenters[index].x}
                        y={finishedCenters[index].y}
                        stackIndex={index}
                        finished
                      />
                    ))}
                  </g>
                )}

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

              <svg
                className="desert-placement-layer"
                viewBox={`0 0 ${BOARD_VIEWBOX.width} ${BOARD_VIEWBOX.height}`}
                role="group"
                aria-label="Desert tile placement"
                data-desert-placement="true"
              >
                {BOARD_SPACES.map((space) => {
                  const legal = canPlaceDesert && legalDesertSpaces.includes(space.space);

                  return (
                    <g
                      key={space.space}
                      data-desert-field={space.space}
                      className={legal ? "desert-field desert-field--legal" : "desert-field"}
                      aria-hidden={legal ? undefined : true}
                      role={legal ? "button" : undefined}
                      tabIndex={legal ? 0 : undefined}
                      aria-label={legal ? `Place a desert tile on space ${space.space}` : undefined}
                      data-desert-place-space={legal ? space.space : undefined}
                      onClick={legal ? () => openDesertDialog(space.space) : undefined}
                      onKeyDown={
                        legal
                          ? (event) => {
                              if (event.key === "Enter" || event.key === " ") {
                                event.preventDefault();
                                openDesertDialog(space.space);
                              }
                            }
                          : undefined
                      }
                    >
                      {legal && (
                        <>
                          <rect
                            x={space.x - space.width / 2}
                            y={space.y - space.height / 2}
                            width={space.width}
                            height={space.height}
                            className="desert-field-hit"
                          />
                          <g
                            transform={`translate(${space.x + space.width / 2 - DESERT_PLUS_INSET} ${
                              space.y - space.height / 2 + DESERT_PLUS_INSET
                            })`}
                            className="desert-add-icon"
                            aria-hidden="true"
                          >
                            <path
                              d={`M ${-DESERT_PLUS_ARM} 0 H ${DESERT_PLUS_ARM} M 0 ${-DESERT_PLUS_ARM} V ${DESERT_PLUS_ARM}`}
                            />
                          </g>
                        </>
                      )}
                    </g>
                  );
                })}
              </svg>

              <button
                ref={addButtonRef}
                type="button"
                className="add-die-button"
                aria-label="Add die"
                title="Add die"
                data-add-die="true"
                style={addButtonStyle}
                onClick={openDiceDialog}
                disabled={!canAddDie}
              >
                <span aria-hidden="true">+</span>
              </button>
            </div>
          </div>

          <Cockpit evaluation={cockpit} error={cockpitError} diceAvailable={canAddDie} />

          <div className="dice-controls" data-dice-controls="true">
            <button
              ref={clearButtonRef}
              type="button"
              className="dice-control-button clear-button"
              onClick={clearBoard}
              disabled={!canClear}
            >
              Clear
            </button>
            <button
              ref={nextRoundButtonRef}
              type="button"
              className="dice-control-button next-round-button"
              onClick={startNextRound}
              disabled={!canStartNextRound}
            >
              Next round
            </button>
          </div>
        </div>

        <p className="sr-only" role="status" aria-live="polite" data-dice-status="true">
          {diceStatus} {camelStatus} {desertStatus}
        </p>
      </div>

      {isDialogOpen && (
        <div
          className="dice-dialog-backdrop"
          onMouseDown={(event) => {
            if (event.button === 0 && event.target === event.currentTarget) {
              closeDialog();
            }
          }}
          onKeyDown={handleDialogKeyDown}
        >
          <div
            ref={dialogRef}
            className={`dice-dialog${dialog?.kind === "desert" ? " desert-dialog" : ""}`}
            role="dialog"
            aria-modal="true"
            aria-labelledby={
              dialog?.kind === "desert" ? "desert-dialog-title" : "dice-dialog-title"
            }
            tabIndex={-1}
          >
            {dialog?.kind === "desert" ? (
              <>
                <h2 id="desert-dialog-title">Place a desert tile</h2>
                <p className="desert-dialog-hint">Choose a face for space {dialog.space}.</p>
                <div className="desert-dialog-choices">
                  {(["oasis", "mirage"] as const).map((kind) => (
                    <button
                      key={kind}
                      type="button"
                      className="desert-choice"
                      aria-label={`Place ${kind} on space ${dialog.space}`}
                      onClick={() => placeDesert(kind)}
                    >
                      <svg className="desert-choice-art" viewBox="0 0 100 100" aria-hidden="true">
                        <rect width="100" height="100" rx="10" className="desert-tile-surface" />
                        <DesertArtwork kind={kind} />
                      </svg>
                      <span>{kind === "oasis" ? "Oasis" : "Mirage"}</span>
                    </button>
                  ))}
                </div>
                <div className="dice-dialog-actions">
                  <button
                    type="button"
                    className="dialog-button dialog-button--secondary"
                    onClick={() => closeDialog()}
                  >
                    Cancel
                  </button>
                </div>
              </>
            ) : (
              <>
                <h2 id="dice-dialog-title">Add die</h2>

                <div className="dice-dialog-field">
                  <label htmlFor="dice-color">Colour</label>
                  <select
                    id="dice-color"
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
              </>
            )}
          </div>
        </div>
      )}
    </main>
  );
}
