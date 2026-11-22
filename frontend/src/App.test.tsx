import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { BOARD_SPACES } from "./boardLayout";

const getDice = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-die]"),
];

const getDiceColors = (container: HTMLElement) => getDice(container).map((die) => die.dataset.die);

const getDiceValues = (container: HTMLElement) =>
  getDice(container).map((die) => die.getAttribute("data-value"));

const getDieSlots = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-die-slot]"),
];

const getCamels = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-camel]"),
];

const getCamelInfo = (container: HTMLElement) =>
  getCamels(container).map((camel) => ({
    color: camel.dataset.camel,
    space: camel.dataset.space ?? null,
    index: Number(camel.dataset.stackIndex),
    finished: camel.dataset.finished === "true",
    transform: camel.getAttribute("transform"),
  }));

const getSelectOptions = (label: string) =>
  [...screen.getByLabelText(label).querySelectorAll("option")]
    .map((option) => option.value)
    .filter(Boolean);

const openDiceDialog = () => {
  fireEvent.click(screen.getByRole("button", { name: "Add die" }));
};

const chooseDie = (color: string, value: string) => {
  fireEvent.change(screen.getByLabelText("Colour"), { target: { value: color } });
  fireEvent.change(screen.getByLabelText("Value"), { target: { value } });
};

const addDie = (color: string, value: string) => {
  openDiceDialog();
  chooseDie(color, value);
  fireEvent.click(screen.getByRole("button", { name: "OK" }));
};

const DIE_COLORS = ["white", "yellow", "orange", "green", "blue"] as const;

const getBetCards = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-leg-bet-card]"),
];

const getAvailableBetCards = (container: HTMLElement) =>
  getBetCards(container).filter((card) => card.dataset.legBetAvailable === "true");

const getBetCardsFor = (container: HTMLElement, color: string) =>
  getBetCards(container).filter((card) => card.dataset.legBetCard === color);

const takeTopBet = (label: string) =>
  fireEvent.click(screen.getByRole("button", { name: `Take ${label} leg-bet tile` }));

const getDesertPlaceButtons = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-desert-place-space]"),
];

const getPlacedDeserts = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-desert]"),
];

const openDesertDialogFor = (space: number) =>
  fireEvent.click(screen.getByRole("button", { name: `Place a desert tile on space ${space}` }));

const fillRound = (values: readonly string[]) => {
  DIE_COLORS.forEach((color, index) => addDie(color, values[index]));
};

/** Roll the five setup dice and start the first leg. */
const completeSetup = () => {
  fillRound(["1", "1", "1", "1", "1"]);
  fireEvent.click(screen.getByRole("button", { name: "Next round" }));
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("App", () => {
  it("renders the single page with the accessible board", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Camel Up board" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Camel Up board" })).toBeInTheDocument();
  });

  it("renders all 16 camel fields", () => {
    const { container } = render(<App />);
    const fields = [...container.querySelectorAll<SVGGElement>("[data-space]")];

    expect(fields).toHaveLength(16);
    expect(fields.map((field) => Number(field.dataset.space))).toEqual(
      Array.from({ length: 16 }, (_, index) => index + 1),
    );
  });

  it("renders field 1 at the top middle and field 16 to its left", () => {
    const { container } = render(<App />);
    const start = container.querySelector('[data-space="1"]');
    const finish = container.querySelector('[data-space="16"]');

    expect(start).toHaveAttribute("data-row", "1");
    expect(start).toHaveAttribute("data-column", "3");
    expect(finish).toHaveAttribute("data-row", "1");
    expect(finish).toHaveAttribute("data-column", "2");
  });

  it("draws the finish line, central panel, empty tray, and five slots", () => {
    const { container } = render(<App />);
    const tray = container.querySelector<SVGRectElement>('[data-dice-tray="true"]');
    const addButton = screen.getByRole("button", { name: "Add die" });

    expect(container.querySelector('[data-finish-line="true"]')).toBeInTheDocument();
    expect(container.querySelector('[data-center-panel="true"]')).toBeInTheDocument();
    expect(container.querySelector('[data-pyramid="true"]')).not.toBeInTheDocument();
    expect(tray).not.toBeNull();
    expect(tray).toHaveAttribute("x", "200");
    expect(tray).toHaveAttribute("y", "400");
    expect(tray).toHaveAttribute("width", "600");
    expect(tray).toHaveAttribute("height", "200");
    expect(getDieSlots(container)).toHaveLength(5);
    expect(getDice(container)).toHaveLength(0);
    expect(addButton).toBeInTheDocument();
    expect(addButton).toHaveStyle({ left: "76.6%", top: "56.8%" });
    expect(addButton.style.getPropertyValue("--add-die-size")).toBe("3cqw");
  });

  it("starts empty with Add die enabled and Clear disabled", () => {
    render(<App />);

    expect(screen.getByRole("button", { name: "Add die" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Clear" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next round" })).toBeDisabled();
    expect(screen.queryByRole("button", { name: /roll/i })).not.toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("No dice");
  });

  it("opens the add-die dialog with empty fields and a disabled OK button", async () => {
    const { container } = render(<App />);

    expect(screen.queryByRole("dialog", { name: "Add die" })).not.toBeInTheDocument();

    openDiceDialog();

    expect(screen.getByRole("dialog", { name: "Add die" })).toBeInTheDocument();
    expect(screen.getByLabelText("Colour")).toHaveValue("");
    expect(screen.getByLabelText("Value")).toHaveValue("");
    expect(screen.getByRole("button", { name: "OK" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Randomize selection" })).toBeEnabled();
    expect(
      screen.getByRole("button", { name: "Randomize selection" }).querySelector("svg"),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeEnabled();
    expect(container.querySelector('[data-page-content="true"]')).toHaveAttribute("inert");
    expect(document.body.style.overflow).toBe("hidden");
    await waitFor(() => expect(screen.getByLabelText("Colour")).toHaveFocus());
  });

  it("offers every remaining colour and only the die values 1 to 3", () => {
    render(<App />);

    openDiceDialog();

    expect(getSelectOptions("Colour")).toEqual(["white", "yellow", "orange", "green", "blue"]);
    expect(getSelectOptions("Value")).toEqual(["1", "2", "3"]);
  });

  it("adds the chosen die to the next slot, closes the dialog, and reports it", async () => {
    const { container } = render(<App />);

    openDiceDialog();
    chooseDie("green", "2");
    expect(screen.getByRole("button", { name: "OK" })).toBeEnabled();

    fireEvent.click(screen.getByRole("button", { name: "OK" }));

    expect(screen.queryByRole("dialog", { name: "Add die" })).not.toBeInTheDocument();
    const dice = getDice(container);
    expect(dice).toHaveLength(1);
    expect(dice[0]).toHaveAttribute("data-die", "green");
    expect(dice[0]).toHaveAttribute("data-value", "2");
    expect(container.querySelector('[data-die-slot="1"] [data-die]')).toBe(dice[0]);
    expect(container.querySelectorAll(".die-slot")).toHaveLength(4);
    expect(screen.getByRole("status")).toHaveTextContent("Dice: Green 2.");
    await waitFor(() => expect(screen.getByRole("button", { name: "Add die" })).toHaveFocus());
  });

  it("closes the dialog on Cancel without adding a die", async () => {
    const { container } = render(<App />);

    openDiceDialog();
    chooseDie("blue", "1");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(screen.queryByRole("dialog", { name: "Add die" })).not.toBeInTheDocument();
    expect(getDice(container)).toHaveLength(0);
    expect(container.querySelector('[data-page-content="true"]')).not.toHaveAttribute("inert");
    expect(document.body.style.overflow).toBe("");
    await waitFor(() => expect(screen.getByRole("button", { name: "Add die" })).toHaveFocus());
  });

  it("traps Tab focus inside the dialog", async () => {
    render(<App />);

    openDiceDialog();

    const colour = screen.getByLabelText("Colour");
    const cancel = screen.getByRole("button", { name: "Cancel" });

    await waitFor(() => expect(colour).toHaveFocus());

    fireEvent.keyDown(colour, { key: "Tab", shiftKey: true });
    expect(cancel).toHaveFocus();

    fireEvent.keyDown(cancel, { key: "Tab" });
    expect(colour).toHaveFocus();

    chooseDie("green", "2");

    const ok = screen.getByRole("button", { name: "OK" });

    expect(ok).toBeEnabled();
    ok.focus();
    fireEvent.keyDown(ok, { key: "Tab" });
    expect(colour).toHaveFocus();

    fireEvent.keyDown(colour, { key: "Tab", shiftKey: true });
    expect(ok).toHaveFocus();
  });

  it("does not close when the dialog itself is clicked", () => {
    render(<App />);

    openDiceDialog();
    fireEvent.click(screen.getByRole("dialog", { name: "Add die" }));

    expect(screen.getByRole("dialog", { name: "Add die" })).toBeInTheDocument();
  });

  it("keeps throw order and offers only colours not yet thrown", () => {
    const { container } = render(<App />);

    addDie("blue", "3");
    openDiceDialog();

    expect(getSelectOptions("Colour")).toEqual(["white", "yellow", "orange", "green"]);

    chooseDie("white", "1");
    fireEvent.click(screen.getByRole("button", { name: "OK" }));

    expect(getDiceColors(container)).toEqual(["blue", "white"]);
    expect(getDiceValues(container)).toEqual(["3", "1"]);
    expect(container.querySelector('[data-die-slot="1"] [data-die]')).toHaveAttribute(
      "data-die",
      "blue",
    );
    expect(container.querySelector('[data-die-slot="2"] [data-die]')).toHaveAttribute(
      "data-die",
      "white",
    );
  });

  it("resets both selectors when the dialog is reopened", () => {
    render(<App />);

    addDie("orange", "2");
    openDiceDialog();

    expect(screen.getByLabelText("Colour")).toHaveValue("");
    expect(screen.getByLabelText("Value")).toHaveValue("");
    expect(screen.getByRole("button", { name: "OK" })).toBeDisabled();
  });

  it("fills both selectors from the random action without adding a die", () => {
    const random = vi.spyOn(Math, "random").mockReturnValueOnce(0).mockReturnValueOnce(0.999);
    const { container } = render(<App />);

    openDiceDialog();
    fireEvent.click(screen.getByRole("button", { name: "Randomize selection" }));

    expect(screen.getByLabelText("Colour")).toHaveValue("white");
    expect(screen.getByLabelText("Value")).toHaveValue("3");
    expect(screen.getByRole("button", { name: "OK" })).toBeEnabled();
    expect(getDice(container)).toHaveLength(0);
    expect(random).toHaveBeenCalledTimes(2);

    fireEvent.click(screen.getByRole("button", { name: "OK" }));

    expect(getDiceColors(container)).toEqual(["white"]);
    expect(getDiceValues(container)).toEqual(["3"]);
  });

  it("keeps the tray empty until OK is pressed", () => {
    const { container } = render(<App />);

    openDiceDialog();
    fireEvent.change(screen.getByLabelText("Colour"), { target: { value: "yellow" } });

    expect(getDice(container)).toHaveLength(0);
    expect(screen.getByRole("button", { name: "OK" })).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Value"), { target: { value: "2" } });

    expect(getDice(container)).toHaveLength(0);
    expect(screen.getByRole("button", { name: "OK" })).toBeEnabled();

    fireEvent.click(screen.getByRole("button", { name: "OK" }));

    expect(getDice(container)).toHaveLength(1);
  });

  it("stops at five dice and moves focus to Next round", async () => {
    const { container } = render(<App />);
    const addButton = screen.getByRole("button", { name: "Add die" });

    addDie("white", "1");
    addDie("yellow", "2");
    addDie("orange", "3");
    addDie("green", "1");
    addDie("blue", "2");

    expect(getDiceColors(container)).toEqual(["white", "yellow", "orange", "green", "blue"]);
    expect(getDiceValues(container)).toEqual(["1", "2", "3", "1", "2"]);
    expect(addButton).toBeDisabled();
    expect(screen.getByRole("button", { name: "Clear" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Next round" })).toBeEnabled();

    await waitFor(() => expect(screen.getByRole("button", { name: "Next round" })).toHaveFocus());

    fireEvent.click(addButton);

    expect(screen.queryByRole("dialog", { name: "Add die" })).not.toBeInTheDocument();
    expect(getDice(container)).toHaveLength(5);
  });

  it("can clear a full tray and start adding again", () => {
    const { container } = render(<App />);
    const addButton = screen.getByRole("button", { name: "Add die" });

    addDie("white", "1");
    addDie("yellow", "2");
    addDie("orange", "3");
    addDie("green", "1");
    addDie("blue", "2");

    expect(addButton).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));

    expect(getDice(container)).toHaveLength(0);
    expect(addButton).toBeEnabled();

    addDie("white", "1");

    expect(getDiceColors(container)).toEqual(["white"]);
  });

  it("clears every die and re-enables Add die", async () => {
    const { container } = render(<App />);

    addDie("white", "1");
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));

    expect(getDice(container)).toHaveLength(0);
    expect(container.querySelectorAll(".die-slot")).toHaveLength(5);
    expect(screen.getByRole("button", { name: "Clear" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Add die" })).toBeEnabled();
    expect(screen.getByRole("status")).toHaveTextContent("No dice");
    await waitFor(() => expect(screen.getByRole("button", { name: "Add die" })).toHaveFocus());
  });

  it("closes the dialog on Escape and returns focus to Add die", async () => {
    const { container } = render(<App />);
    const addButton = screen.getByRole("button", { name: "Add die" });

    openDiceDialog();
    chooseDie("green", "2");
    fireEvent.keyDown(document, { key: "Escape" });

    expect(screen.queryByRole("dialog", { name: "Add die" })).not.toBeInTheDocument();
    expect(getDice(container)).toHaveLength(0);
    await waitFor(() => expect(addButton).toHaveFocus());
  });

  it("closes the dialog on a primary backdrop press and ignores secondary presses", async () => {
    const { container } = render(<App />);

    openDiceDialog();
    chooseDie("orange", "3");
    const backdrop = container.querySelector(".dice-dialog-backdrop");

    expect(backdrop).not.toBeNull();
    fireEvent.mouseDown(backdrop as Element, { button: 2 });
    expect(screen.getByRole("dialog", { name: "Add die" })).toBeInTheDocument();

    fireEvent.mouseDown(backdrop as Element, { button: 0 });

    expect(screen.queryByRole("dialog", { name: "Add die" })).not.toBeInTheDocument();
    expect(getDice(container)).toHaveLength(0);
    await waitFor(() => expect(screen.getByRole("button", { name: "Add die" })).toHaveFocus());
  });

  it("renders small numbers only for the twelve non-corner fields", () => {
    const { container } = render(<App />);
    const numbers = [...container.querySelectorAll<SVGTextElement>("[data-number]")];

    expect(numbers).toHaveLength(12);
    expect(numbers.map((number) => Number(number.dataset.number))).toEqual([
      1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16,
    ]);

    for (const corner of [3, 7, 11, 15]) {
      expect(container.querySelector(`[data-space="${corner}"] [data-number]`)).toBeNull();
    }
  });

  it("turns the numbers so their tops point outwards", () => {
    const { container } = render(<App />);
    const rotations = new Map([
      [1, "0"],
      [2, "0"],
      [4, "90"],
      [5, "90"],
      [6, "90"],
      [8, "180"],
      [9, "180"],
      [10, "180"],
      [12, "-90"],
      [13, "-90"],
      [14, "-90"],
      [16, "0"],
    ]);

    for (const [space, rotation] of rotations) {
      expect(container.querySelector(`[data-number="${space}"]`)).toHaveAttribute(
        "data-rotation",
        rotation,
      );
    }
  });

  it("uses the same tile class for every field and has no start or finish captions", () => {
    const { container } = render(<App />);

    expect(container.querySelectorAll(".space-tile")).toHaveLength(16);
    expect(
      container.querySelector(".space-tile--start, .space-tile--finish, .space-tile--alternate"),
    ).toBeNull();
    expect(screen.queryByText("START")).not.toBeInTheDocument();
    expect(screen.queryByText("ZIEL")).not.toBeInTheDocument();
  });

  it("uses English UI text only", () => {
    render(<App />);

    for (const text of ["DAS SPIELFELD", "Spielfeld", "Würfel", "Zufällig"]) {
      expect(screen.queryByText(text)).not.toBeInTheDocument();
    }

    expect(screen.getByRole("button", { name: "Add die" })).toBeInTheDocument();
    expect(screen.getByText("Clear")).toBeInTheDocument();

    openDiceDialog();

    expect(screen.getByRole("dialog", { name: "Add die" })).toBeInTheDocument();
    expect(screen.getByText("Colour")).toBeInTheDocument();
    expect(screen.getByText("Value")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Randomize selection" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "OK" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Next round" })).toBeInTheDocument();
  });

  it("starts with no camels on the field", () => {
    const { container } = render(<App />);

    expect(getCamels(container)).toHaveLength(0);
    expect(container.querySelector('[data-camels="true"]')).toBeNull();
    expect(screen.getByRole("status")).toHaveTextContent("No camels");
  });

  it("places a new camel at the die value, centred in the field", () => {
    const { container } = render(<App />);

    addDie("white", "1");

    expect(getCamelInfo(container)).toEqual([
      {
        color: "white",
        space: "1",
        index: 0,
        finished: false,
        transform: "translate(500 100)",
      },
    ]);
  });

  it("stacks camels tightly with the first one below the newcomer", () => {
    const { container } = render(<App />);

    addDie("white", "1");
    addDie("yellow", "1");

    const camels = getCamelInfo(container);

    expect(camels.map(({ color, index }) => [color, index])).toEqual([
      ["white", 0],
      ["yellow", 1],
    ]);
    expect(camels[0].transform).toBe("translate(500 119)");
    expect(camels[1].transform).toBe("translate(500 81)");
  });

  it("keeps the camel shape unchanged when other camels stack on it", () => {
    const { container } = render(<App />);

    addDie("white", "1");
    const whiteBefore = getCamels(container)[0].innerHTML;

    addDie("yellow", "1");
    addDie("orange", "1");

    expect(getCamels(container)[0].innerHTML).toBe(whiteBefore);
  });

  it("enables Next round only with five dice, then clears dice but keeps camels", () => {
    const { container } = render(<App />);
    const nextRound = screen.getByRole("button", { name: "Next round" });

    addDie("white", "1");
    addDie("yellow", "1");
    addDie("orange", "1");
    addDie("green", "1");
    expect(nextRound).toBeDisabled();

    addDie("blue", "1");
    expect(nextRound).toBeEnabled();

    fireEvent.click(nextRound);

    expect(getDice(container)).toHaveLength(0);
    expect(getCamelInfo(container).map(({ color }) => color)).toEqual([
      "white",
      "yellow",
      "orange",
      "green",
      "blue",
    ]);
    expect(nextRound).toBeDisabled();
    expect(screen.getByRole("button", { name: "Clear" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Add die" })).toBeEnabled();
  });

  it("moves a camel and its passengers when its die is thrown in the next round", () => {
    const { container } = render(<App />);

    addDie("white", "1");
    addDie("yellow", "1");
    addDie("orange", "2");
    addDie("green", "2");
    addDie("blue", "3");
    fireEvent.click(screen.getByRole("button", { name: "Next round" }));

    addDie("white", "1");

    expect(getCamelInfo(container).map(({ color, space, index }) => [color, space, index])).toEqual(
      [
        ["orange", "2", 0],
        ["green", "2", 1],
        ["white", "2", 2],
        ["yellow", "2", 3],
        ["blue", "3", 0],
      ],
    );
  });

  it("lets Clear remove every camel and die", () => {
    const { container } = render(<App />);

    addDie("white", "1");
    addDie("yellow", "1");
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));

    expect(getCamels(container)).toHaveLength(0);
    expect(getDice(container)).toHaveLength(0);
    expect(screen.getByRole("button", { name: "Clear" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next round" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Add die" })).toBeEnabled();
  });

  it("draws a camel that crosses the goal above the finish and locks the round", async () => {
    const { container } = render(<App />);

    for (let round = 0; round < 5; round += 1) {
      fillRound(["3", "1", "1", "1", "1"]);
      fireEvent.click(screen.getByRole("button", { name: "Next round" }));
    }

    expect(getCamelInfo(container).find(({ color }) => color === "white")?.space).toBe("15");

    addDie("white", "2");

    const finished = getCamelInfo(container).filter(({ finished: isFinished }) => isFinished);

    expect(finished).toHaveLength(1);
    expect(finished[0]).toMatchObject({
      color: "white",
      space: null,
      transform: "translate(400 -19)",
    });
    expect(screen.getByRole("button", { name: "Add die" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next round" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("Race finished");
    await waitFor(() => expect(screen.getByRole("button", { name: "Clear" })).toHaveFocus());

    fireEvent.click(screen.getByRole("button", { name: "Clear" }));

    expect(getCamels(container)).toHaveLength(0);
    expect(getDice(container)).toHaveLength(0);
    expect(screen.getByRole("button", { name: "Add die" })).toBeEnabled();
  }, 20000);

  it("focuses Clear when the fifth die of a round finishes the race", async () => {
    const { container } = render(<App />);

    for (let round = 0; round < 5; round += 1) {
      fillRound(["3", "1", "1", "1", "1"]);
      fireEvent.click(screen.getByRole("button", { name: "Next round" }));
    }

    addDie("yellow", "1");
    addDie("orange", "1");
    addDie("green", "1");
    addDie("blue", "1");
    addDie("white", "2");

    expect(getCamelInfo(container).filter(({ finished: isFinished }) => isFinished)).toHaveLength(
      1,
    );
    expect(screen.getByRole("button", { name: "Add die" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Next round" })).toBeDisabled();
    await waitFor(() => expect(screen.getByRole("button", { name: "Clear" })).toHaveFocus());
  }, 20000);

  describe("leg-bet cards", () => {
    it("exposes the cards outside the board's role=img SVG", () => {
      const { container } = render(<App />);
      const board = screen.getByRole("img", { name: "Camel Up board" });
      const bettingField = container.querySelector(".betting-field");
      const whiteFive = screen.getByRole("button", { name: "Take White 5 leg-bet tile" });

      expect(board.contains(whiteFive)).toBe(false);
      expect(bettingField).toContainElement(whiteFive);
      expect(bettingField?.querySelector("defs #card-shadow")).toBeInTheDocument();
      expect(board.querySelector("#card-shadow")).toBeNull();
      expect(screen.getByRole("group", { name: "Leg-betting tiles" })).toBeInTheDocument();

      const gameStyle = container.querySelector(".game")?.getAttribute("style");
      expect(gameStyle).toContain("--bet-scale: 1.19");
      expect(gameStyle).toContain("--bet-band-aspect: 1000 / 190");
    });

    it("starts with three cards per camel but none available during setup", () => {
      const { container } = render(<App />);

      expect(container.querySelector('[data-leg-bets="true"]')).toBeInTheDocument();
      expect(getBetCards(container)).toHaveLength(15);
      expect(getAvailableBetCards(container)).toHaveLength(0);
      expect(container.querySelectorAll("[data-leg-bet-empty]")).toHaveLength(0);
      for (const label of ["White", "Yellow", "Orange", "Green", "Blue"]) {
        expect(
          screen.getByRole("button", { name: `Take ${label} 5 leg-bet tile` }),
        ).toHaveAttribute("aria-disabled", "true");
      }

      // Covered tiles are painted first, so the top tile is the last sibling.
      const whiteCards = getBetCardsFor(container, "white");
      expect(whiteCards.map((card) => card.dataset.legBetValue)).toEqual(["2", "3", "5"]);
      expect(whiteCards[whiteCards.length - 1]).not.toHaveAttribute("data-leg-bet-available");
      expect(getCamels(container)).toHaveLength(0);
    });

    it("enables the top cards once the setup dice are done", () => {
      const { container } = render(<App />);

      fillRound(["1", "1", "1", "1", "1"]);
      expect(getAvailableBetCards(container)).toHaveLength(0);

      fireEvent.click(screen.getByRole("button", { name: "Next round" }));

      expect(getAvailableBetCards(container).map((card) => card.dataset.legBetCard)).toEqual([
        ...DIE_COLORS,
      ]);
      expect(screen.getByRole("button", { name: "Take White 5 leg-bet tile" })).toBeEnabled();
    });

    it("only takes the fully visible top card and reveals the next value", () => {
      const { container } = render(<App />);
      completeSetup();
      const coveredThree = container.querySelector(
        '[data-leg-bet-card="white"][data-leg-bet-value="3"]',
      );

      fireEvent.click(coveredThree as Element);

      expect(getBetCardsFor(container, "white")).toHaveLength(3);
      takeTopBet("White 5");

      expect(
        getBetCardsFor(container, "white")
          .map((card) => card.dataset.legBetValue)
          .sort(),
      ).toEqual(["2", "3"]);
      expect(screen.queryByRole("button", { name: "Take White 5 leg-bet tile" })).toBeNull();
      expect(screen.getByRole("button", { name: "Take White 3 leg-bet tile" })).toBeInTheDocument();
      expect(
        getBetCardsFor(container, "yellow")
          .map((card) => card.dataset.legBetValue)
          .sort(),
      ).toEqual(["2", "3", "5"]);
      expect(getAvailableBetCards(container)).toHaveLength(5);
    });

    it("leaves an empty, non-clickable slot after taking the last card", () => {
      const { container } = render(<App />);
      completeSetup();

      takeTopBet("White 5");
      takeTopBet("White 3");
      takeTopBet("White 2");

      expect(getBetCardsFor(container, "white")).toHaveLength(0);
      expect(container.querySelector('[data-leg-bet-empty="white"]')).toBeInTheDocument();
      expect(container.querySelectorAll("[data-leg-bet-empty]")).toHaveLength(1);
      expect(getBetCards(container)).toHaveLength(12);
      expect(getAvailableBetCards(container)).toHaveLength(4);
      expect(screen.queryByRole("button", { name: /Take White/ })).toBeNull();
    });

    it("keeps keyboard focus on the revealed tile and moves it to Clear when exhausted", async () => {
      render(<App />);
      completeSetup();

      const five = screen.getByRole("button", { name: "Take Blue 5 leg-bet tile" });
      five.focus();
      fireEvent.keyDown(five, { key: "Enter" });

      const three = screen.getByRole("button", { name: "Take Blue 3 leg-bet tile" });
      expect(three).toHaveFocus();
      fireEvent.keyDown(three, { key: " " });

      const two = screen.getByRole("button", { name: "Take Blue 2 leg-bet tile" });
      expect(two).toHaveFocus();
      fireEvent.keyDown(two, { key: "Enter" });

      await waitFor(() => expect(screen.getByRole("button", { name: "Clear" })).toHaveFocus());
      expect(screen.queryByRole("button", { name: /Take Blue/ })).toBeNull();
    });

    it("disables every top card once five dice are thrown", () => {
      const { container } = render(<App />);
      completeSetup();

      expect(getAvailableBetCards(container)).toHaveLength(5);
      fillRound(["1", "1", "1", "1", "1"]);

      for (const label of ["White", "Yellow", "Orange", "Green", "Blue"]) {
        const card = screen.getByRole("button", { name: `Take ${label} 5 leg-bet tile` });
        expect(card).toHaveAttribute("aria-disabled", "true");
        fireEvent.click(card);
        fireEvent.keyDown(card, { key: "Enter" });
      }

      expect(getBetCards(container)).toHaveLength(15);
      expect(getAvailableBetCards(container)).toHaveLength(0);
    }, 20000);

    it("disables the cards when a camel finishes the race early", () => {
      const { container } = render(<App />);

      for (let round = 0; round < 5; round += 1) {
        fillRound(["3", "1", "1", "1", "1"]);
        fireEvent.click(screen.getByRole("button", { name: "Next round" }));
      }
      addDie("white", "2");

      const card = screen.getByRole("button", { name: "Take Yellow 5 leg-bet tile" });
      expect(card).toHaveAttribute("aria-disabled", "true");
      fireEvent.click(card);

      expect(getBetCards(container)).toHaveLength(15);
    }, 20000);

    it("resets all stacks when Clear is pressed, even if only a card was taken", async () => {
      const { container } = render(<App />);
      completeSetup();
      const clear = screen.getByRole("button", { name: "Clear" });

      takeTopBet("White 5");
      expect(clear).toBeEnabled();

      fireEvent.click(clear);

      // Clear returns to setup, so the cards are disabled again.
      expect(getBetCards(container)).toHaveLength(15);
      expect(getAvailableBetCards(container)).toHaveLength(0);
      expect(screen.getByRole("button", { name: "Take White 5 leg-bet tile" })).toHaveAttribute(
        "aria-disabled",
        "true",
      );
      expect(clear).toBeDisabled();
      await waitFor(() => {
        const lastCall = vi.mocked(fetch).mock.calls.at(-1);
        const body = JSON.parse(lastCall?.[1]?.body as string) as { setupRound: boolean };
        expect(body.setupRound).toBe(true);
      });
      expect(container.querySelector('[data-action="legBet"]')).toHaveAttribute(
        "data-leg-bet-available",
        "false",
      );
    });

    it("resets all stacks when Next round is pressed", () => {
      const { container } = render(<App />);
      completeSetup();

      takeTopBet("Orange 5");
      fillRound(["1", "1", "1", "1", "1"]);
      expect(
        screen.getByRole("button", { name: "Take Orange 3 leg-bet tile" }),
      ).toBeInTheDocument();

      fireEvent.click(screen.getByRole("button", { name: "Next round" }));

      expect(getBetCards(container)).toHaveLength(15);
      expect(getAvailableBetCards(container)).toHaveLength(5);
      expect(
        screen.getByRole("button", { name: "Take Orange 5 leg-bet tile" }),
      ).toBeInTheDocument();
    });
  });

  it("renders the cockpit below the board and refreshes it when a die is added", async () => {
    const { container } = render(<App />);
    const board = container.querySelector<HTMLElement>(".board-stack");
    const cockpit = container.querySelector<HTMLElement>('[data-cockpit="true"]');

    expect(board).toBeInTheDocument();
    expect(cockpit).toBeInTheDocument();
    expect(board?.nextElementSibling).toBe(cockpit);

    const fetchMock = vi.mocked(fetch);
    fetchMock.mockClear();
    addDie("white", "2");

    await waitFor(() => expect(fetchMock).toHaveBeenCalled());
    const lastCall = fetchMock.mock.calls.at(-1);
    const body = JSON.parse(lastCall?.[1]?.body as string) as {
      thrownDice: string[];
      stacks: Record<string, string[]>;
      setupRound: boolean;
    };

    expect(body.thrownDice).toEqual(["white"]);
    expect(body.stacks).toEqual({ "2": ["white"] });
    expect(body.setupRound).toBe(true);
  });

  it("shows both cockpit placeholders during setup", async () => {
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockClear();
    const { container } = render(<App />);

    await waitFor(() => expect(fetchMock).toHaveBeenCalled());

    expect(container.querySelector('[data-action="dice"] [data-expected-coins]')).toHaveTextContent(
      "—",
    );
    expect(container.querySelector('[data-action="legBet"]')).toHaveAttribute(
      "data-leg-bet-available",
      "false",
    );
  });

  it("shows leg bets as available once the setup round is over", async () => {
    const { container } = render(<App />);
    completeSetup();

    await waitFor(() =>
      expect(container.querySelector('[data-action="legBet"]')).toHaveAttribute(
        "data-leg-bet-available",
        "true",
      ),
    );
    const fetchMock = vi.mocked(fetch);
    await waitFor(() => expect(fetchMock).toHaveBeenCalled());
    const lastCall = fetchMock.mock.calls.at(-1);
    const body = JSON.parse(lastCall?.[1]?.body as string) as { setupRound: boolean };

    expect(body.setupRound).toBe(false);
  });

  it("shows the cockpit backend error while keeping the setup placeholder", async () => {
    vi.mocked(fetch).mockRejectedValueOnce(new TypeError("Failed to fetch"));
    const { container } = render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent(/Rust backend/);
    expect(container.querySelector('[data-action="legBet"]')).toHaveAttribute(
      "data-leg-bet-available",
      "false",
    );
    expect(container.querySelector('[data-action="legBet"]')).toHaveTextContent("—");
    expect(container.querySelectorAll("[data-cockpit-camel]")).toHaveLength(0);
  });

  it("surfaces a backend validation error instead of startup instructions", async () => {
    vi.mocked(fetch).mockResolvedValueOnce({
      ok: false,
      status: 400,
      json: async () => ({ error: "space 0 must be in 1..=16" }),
    } as Response);
    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent("space 0 must be in 1..=16");
    expect(screen.getByRole("alert")).not.toHaveTextContent("cargo run");
  });

  it("clears the previous evaluation while a new one is in flight", async () => {
    let resolvePending!: (response: Response) => void;
    const pending = new Promise<Response>((resolve) => {
      resolvePending = resolve;
    });
    const evaluation = {
      ok: true,
      status: 200,
      json: async () => ({ dice: { expectedCoins: 1 }, legBet: { available: true, camels: [] } }),
    } as Response;
    const fetchMock = vi.mocked(fetch);
    fetchMock.mockImplementationOnce(async () => evaluation).mockImplementationOnce(() => pending);

    const { container } = render(<App />);
    const ticketValue = () => container.querySelector('[data-action="dice"] [data-expected-coins]');
    await waitFor(() => expect(ticketValue()).toHaveTextContent("1.00"));

    addDie("white", "2");

    expect(ticketValue()).toHaveTextContent("—");
    resolvePending(evaluation);
    await waitFor(() => expect(ticketValue()).toHaveTextContent("1.00"));
  });

  it("blanks the ticket value once all five dice are thrown", () => {
    const { container } = render(<App />);
    const ticketValue = () => container.querySelector('[data-action="dice"] [data-expected-coins]');

    fillRound(["1", "2", "3", "1", "2"]);

    expect(ticketValue()).toHaveTextContent("—");
  });

  describe("desert tile placement", () => {
    it("offers only legal fields once a leg has started", () => {
      const { container } = render(<App />);

      expect(screen.getByRole("group", { name: "Desert tile placement" })).toBeInTheDocument();
      expect(getDesertPlaceButtons(container)).toHaveLength(0);

      completeSetup();
      const spaces = getDesertPlaceButtons(container).map((field) =>
        Number(field.dataset.desertPlaceSpace),
      );

      expect(spaces).toEqual(Array.from({ length: 15 }, (_, index) => index + 2));

      addDie("white", "1");
      // The white unit and its passengers now stand on space 2.
      expect(
        getDesertPlaceButtons(container).some((field) => field.dataset.desertPlaceSpace === "2"),
      ).toBe(false);

      (["yellow", "orange", "green", "blue"] as const).forEach((color) => addDie(color, "1"));
      expect(getDesertPlaceButtons(container)).toHaveLength(0);
    }, 20000);

    it("shows only a small plus at the top right of legal fields", () => {
      const { container } = render(<App />);
      completeSetup();
      const field = getDesertPlaceButtons(container)[0];
      const space = Number(field.dataset.desertPlaceSpace);
      const layout = BOARD_SPACES.find((item) => item.space === space);
      const icon = field.querySelector<SVGGElement>(".desert-add-icon");
      const translate = icon?.getAttribute("transform")?.match(/translate\(([-\d.]+) ([-\d.]+)\)/);

      expect(layout).toBeDefined();
      expect(translate).not.toBeNull();
      expect(Number(translate?.[1])).toBeGreaterThan(layout?.x ?? 0);
      expect(Number(translate?.[2])).toBeLessThan(layout?.y ?? 0);
      expect(icon?.querySelector("circle")).not.toBeInTheDocument();
      expect(icon?.querySelectorAll("path")).toHaveLength(1);
    });

    it("places a flat oasis without a badge behind the camels and reports it", async () => {
      const { container } = render(<App />);
      completeSetup();

      openDesertDialogFor(5);

      expect(screen.getByRole("dialog", { name: "Place a desert tile" })).toBeInTheDocument();
      expect(container.querySelector('[data-page-content="true"]')).toHaveAttribute("inert");
      expect(document.body.style.overflow).toBe("hidden");
      const oasis = screen.getByRole("button", { name: "Place oasis on space 5" });
      const mirage = screen.getByRole("button", { name: "Place mirage on space 5" });
      await waitFor(() => expect(oasis).toHaveFocus());

      fireEvent.click(oasis);

      const tile = getPlacedDeserts(container)[0];
      expect(screen.queryByRole("dialog", { name: "Place a desert tile" })).not.toBeInTheDocument();
      expect(tile).toMatchObject({
        dataset: expect.objectContaining({
          desert: "oasis",
          desertSpace: "5",
        }),
      });
      expect(tile.querySelector(".desert-badge")).not.toBeInTheDocument();
      expect(tile.querySelector("circle")).not.toBeInTheDocument();
      const camels = container.querySelector('[data-camels="true"]');
      expect(tile.compareDocumentPosition(camels as Node) & Node.DOCUMENT_POSITION_FOLLOWING).toBe(
        Node.DOCUMENT_POSITION_FOLLOWING,
      );
      expect(screen.getByRole("status")).toHaveTextContent("oasis on 5");
      expect(screen.getByRole("button", { name: "Clear" })).toBeEnabled();
      expect(
        getDesertPlaceButtons(container).map((field) => field.dataset.desertPlaceSpace),
      ).not.toContain("5");
      expect(mirage).not.toBeInTheDocument();

      await waitFor(() =>
        expect(
          screen.getByRole("button", { name: "Place a desert tile on space 7" }),
        ).toHaveFocus(),
      );
    });

    it("moves a camel one extra space forward over an oasis", () => {
      const { container } = render(<App />);
      completeSetup();
      openDesertDialogFor(2);
      fireEvent.click(screen.getByRole("button", { name: "Place oasis on space 2" }));

      addDie("white", "1");

      expect(getCamelInfo(container).map(({ space }) => space)).toEqual(["3", "3", "3", "3", "3"]);
    });

    it("moves a camel one space back over a mirage", () => {
      const { container } = render(<App />);
      completeSetup();
      openDesertDialogFor(3);
      fireEvent.click(screen.getByRole("button", { name: "Place mirage on space 3" }));

      addDie("white", "2");

      expect(getCamelInfo(container).map(({ space }) => space)).toEqual(["2", "2", "2", "2", "2"]);
    });

    it("sends placed tiles with the API request", async () => {
      const fetchMock = vi.mocked(fetch);
      const { container } = render(<App />);
      completeSetup();
      openDesertDialogFor(6);
      fireEvent.click(screen.getByRole("button", { name: "Place mirage on space 6" }));

      await waitFor(() => {
        const lastCall = fetchMock.mock.calls.at(-1);
        const body = JSON.parse(lastCall?.[1]?.body as string) as {
          deserts: readonly { space: number; kind: string }[];
        };
        expect(body.deserts).toEqual([{ space: 6, kind: "mirage" }]);
      });
      expect(container.querySelector('[data-desert="mirage"]')).toBeInTheDocument();
    });

    it("supports keyboard activation, Escape, backdrop dismissal and focus restoration", async () => {
      const { container } = render(<App />);
      completeSetup();
      const field = screen.getByRole("button", { name: "Place a desert tile on space 8" });

      field.focus();
      fireEvent.keyDown(field, { key: "Enter" });
      expect(screen.getByRole("dialog", { name: "Place a desert tile" })).toBeInTheDocument();

      fireEvent.keyDown(document, { key: "Escape" });
      expect(screen.queryByRole("dialog", { name: "Place a desert tile" })).not.toBeInTheDocument();
      expect(getPlacedDeserts(container)).toHaveLength(0);
      await waitFor(() => expect(field).toHaveFocus());

      openDesertDialogFor(8);
      const backdrop = container.querySelector(".dice-dialog-backdrop");
      fireEvent.mouseDown(backdrop as Element, { button: 0 });
      expect(screen.queryByRole("dialog", { name: "Place a desert tile" })).not.toBeInTheDocument();
      await waitFor(() => expect(field).toHaveFocus());

      field.focus();
      fireEvent.keyDown(field, { key: " " });
      expect(screen.getByRole("dialog", { name: "Place a desert tile" })).toBeInTheDocument();
      fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
      expect(getPlacedDeserts(container)).toHaveLength(0);
      await waitFor(() => expect(field).toHaveFocus());
    });

    it("clears tiles on Clear and on Next round", () => {
      const { container } = render(<App />);
      completeSetup();
      openDesertDialogFor(9);
      fireEvent.click(screen.getByRole("button", { name: "Place oasis on space 9" }));

      fireEvent.click(screen.getByRole("button", { name: "Clear" }));

      expect(getPlacedDeserts(container)).toHaveLength(0);
      expect(getDesertPlaceButtons(container)).toHaveLength(0);

      completeSetup();
      openDesertDialogFor(9);
      fireEvent.click(screen.getByRole("button", { name: "Place oasis on space 9" }));
      fillRound(["1", "1", "1", "1", "1"]);
      fireEvent.click(screen.getByRole("button", { name: "Next round" }));

      expect(getPlacedDeserts(container)).toHaveLength(0);
      expect(getDesertPlaceButtons(container).length).toBeGreaterThan(0);
    }, 20000);
  });
});
