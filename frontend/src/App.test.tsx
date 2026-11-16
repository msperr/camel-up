import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const getDice = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-die]"),
];

const getDiceColors = (container: HTMLElement) => getDice(container).map((die) => die.dataset.die);

const getDiceValues = (container: HTMLElement) =>
  getDice(container).map((die) => die.getAttribute("data-value"));

const getDieSlots = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-die-slot]"),
];

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

  it("stops at five dice and moves focus to Clear", async () => {
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

    await waitFor(() => expect(screen.getByRole("button", { name: "Clear" })).toHaveFocus());

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

  it("closes the dialog when the backdrop is clicked", async () => {
    const { container } = render(<App />);

    openDiceDialog();
    chooseDie("orange", "3");
    const backdrop = container.querySelector(".dice-dialog-backdrop");

    expect(backdrop).not.toBeNull();
    fireEvent.click(backdrop as Element);

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
  });
});
