import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const getDice = (container: HTMLElement) => [
  ...container.querySelectorAll<SVGGElement>("[data-die]"),
];

const getDiceValues = (container: HTMLElement) =>
  getDice(container).map((die) => die.getAttribute("data-value"));

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

  it("draws the finish line, central panel, and a centred dice tray", () => {
    const { container } = render(<App />);
    const tray = container.querySelector<SVGRectElement>('[data-dice-tray="true"]');

    expect(container.querySelector('[data-finish-line="true"]')).toBeInTheDocument();
    expect(container.querySelector('[data-center-panel="true"]')).toBeInTheDocument();
    expect(container.querySelector('[data-pyramid="true"]')).not.toBeInTheDocument();
    expect(tray).not.toBeNull();
    expect(tray).toHaveAttribute("x", "200");
    expect(tray).toHaveAttribute("y", "400");
    expect(tray).toHaveAttribute("width", "600");
    expect(tray).toHaveAttribute("height", "200");
  });

  it("starts with five dice in a fixed colour order and centred row", () => {
    const { container } = render(<App />);
    const dice = getDice(container);

    expect(dice).toHaveLength(5);
    expect(dice.map((die) => die.dataset.die)).toEqual([
      "white",
      "yellow",
      "orange",
      "green",
      "blue",
    ]);
    expect(getDiceValues(container)).toEqual(["1", "2", "3", "2", "1"]);
    expect(dice.map((die) => die.getAttribute("transform"))).toEqual([
      "translate(304 500) rotate(-8)",
      "translate(402 500) rotate(5)",
      "translate(500 500) rotate(-3)",
      "translate(598 500) rotate(7)",
      "translate(696 500) rotate(-6)",
    ]);
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

  it("renders an icon-only Roll button and a Clear button", () => {
    render(<App />);

    const rollButton = screen.getByRole("button", { name: "Roll dice" });
    const clearButton = screen.getByRole("button", { name: "Clear" });

    expect(rollButton.querySelector("svg")).toBeInTheDocument();
    expect(rollButton).toHaveTextContent("");
    expect(clearButton).toBeEnabled();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Dice: White 1, Yellow 2, Orange 3, Green 2, Blue 1.",
    );
  });

  it("clears every die and disables Clear", () => {
    const { container } = render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Clear" }));

    expect(getDice(container)).toHaveLength(0);
    expect(screen.getByRole("button", { name: "Clear" })).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("No dice");
  });

  it("rolls five new values and restores the dice after Clear", () => {
    const random = vi
      .spyOn(Math, "random")
      .mockReturnValueOnce(0)
      .mockReturnValueOnce(0.34)
      .mockReturnValueOnce(0.67)
      .mockReturnValueOnce(0.999)
      .mockReturnValueOnce(0.5);

    const { container } = render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    expect(getDice(container)).toHaveLength(0);

    fireEvent.click(screen.getByRole("button", { name: "Roll dice" }));

    expect(random).toHaveBeenCalledTimes(5);
    expect(getDiceValues(container)).toEqual(["1", "2", "3", "3", "2"]);
    expect(getDice(container).map((die) => die.dataset.die)).toEqual([
      "white",
      "yellow",
      "orange",
      "green",
      "blue",
    ]);
    expect(screen.getByRole("status")).toHaveTextContent(
      "Dice: White 1, Yellow 2, Orange 3, Green 3, Blue 2.",
    );
  });

  it("uses English UI text only", () => {
    render(<App />);

    for (const text of ["DAS SPIELFELD", "Spielfeld", "Würfel", "Zufällig"]) {
      expect(screen.queryByText(text)).not.toBeInTheDocument();
    }

    expect(screen.getByText("Clear")).toBeInTheDocument();
  });
});
