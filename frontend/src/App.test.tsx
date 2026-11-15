import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("App", () => {
  it("renders the single page with the accessible board", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "Camel Up Spielfeld" })).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Camel Up Spielfeld" })).toBeInTheDocument();
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

  it("draws the finish line, central pyramid panel, and five dice", () => {
    const { container } = render(<App />);

    expect(container.querySelector('[data-finish-line="true"]')).toBeInTheDocument();
    expect(container.querySelector('[data-center-panel="true"]')).toBeInTheDocument();
    expect(container.querySelector('[data-pyramid="true"]')).toBeInTheDocument();
    expect(container.querySelectorAll("[data-die]")).toHaveLength(5);
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
});
