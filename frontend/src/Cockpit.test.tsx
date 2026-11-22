import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import Cockpit from "./Cockpit";
import type { CockpitResponse } from "./cockpit";

const evaluation: CockpitResponse = {
  dice: { expectedCoins: 1 },
  legBet: {
    available: true,
    camels: [
      { color: "orange", expectedCoins: 2.345, roundWinProbability: 0.425 },
      { color: "white", expectedCoins: 1.2, roundWinProbability: 0.2 },
      { color: "blue", expectedCoins: null, roundWinProbability: 0.31 },
      { color: "yellow", expectedCoins: 0.9, roundWinProbability: 0.15 },
      { color: "green", expectedCoins: 0.4, roundWinProbability: 0.1 },
    ],
  },
};

const getRow = (container: HTMLElement, action: string) =>
  container.querySelector<HTMLElement>(`[data-action="${action}"]`);

describe("Cockpit", () => {
  it("shows the four rules actions in order", () => {
    const { container } = render(<Cockpit evaluation={evaluation} error={null} />);
    const rows = [...container.querySelectorAll<HTMLElement>("[data-action]")];

    expect(rows.map((row) => row.dataset.action)).toEqual(["dice", "desert", "legBet", "overall"]);
    expect(getRow(container, "dice")).toHaveTextContent("Take a pyramid ticket and roll");
    expect(getRow(container, "desert")).toHaveTextContent("Place or move your desert tile");
    expect(getRow(container, "legBet")).toHaveTextContent("Take a leg-betting tile");
    expect(getRow(container, "overall")).toHaveTextContent("Bet on the overall winner or loser");
  });

  it("shows the guaranteed coin win for rolling without camel columns", () => {
    const { container } = render(<Cockpit evaluation={evaluation} error={null} />);
    const row = getRow(container, "dice");

    expect(row).toHaveTextContent("1.00");
    expect(row?.querySelector("[data-cockpit-camel]")).not.toBeInTheDocument();
    expect(
      getRow(container, "desert")?.querySelector("[data-cockpit-camel]"),
    ).not.toBeInTheDocument();
  });

  it("renders two formatted values per camel in the order supplied by the API", () => {
    const { container } = render(<Cockpit evaluation={evaluation} error={null} />);
    const camels = [...container.querySelectorAll<HTMLElement>("[data-cockpit-camel]")];

    expect(camels.map((camel) => camel.dataset.cockpitCamel)).toEqual([
      "orange",
      "white",
      "blue",
      "yellow",
      "green",
    ]);
    expect(camels[0].querySelector("[data-expected-coins]")).toHaveTextContent("2.35");
    expect(camels[0].querySelector("[data-win-probability]")).toHaveTextContent("43%");
    expect(camels[1].querySelector("[data-expected-coins]")).toHaveTextContent("1.20");
    expect(camels[1].querySelector("[data-win-probability]")).toHaveTextContent("20%");
  });

  it("appends blank camels when an evaluation omits colours", () => {
    const partial: CockpitResponse = {
      ...evaluation,
      legBet: { available: true, camels: [evaluation.legBet.camels[0]] },
    };
    const { container } = render(<Cockpit evaluation={partial} error={null} />);

    expect(
      [...container.querySelectorAll<HTMLElement>("[data-cockpit-camel]")].map(
        (camel) => camel.dataset.cockpitCamel,
      ),
    ).toEqual(["orange", "white", "yellow", "green", "blue"]);
  });

  it("leaves coins blank but shows the win probability for an exhausted stack", () => {
    const { container } = render(<Cockpit evaluation={evaluation} error={null} />);
    const blue = container.querySelector<HTMLElement>('[data-cockpit-camel="blue"]');

    expect(blue?.querySelector("[data-expected-coins]")).toBeEmptyDOMElement();
    expect(blue?.querySelector("[data-win-probability]")).toHaveTextContent("31%");
  });

  it("uses dashes for the unimplemented actions", () => {
    const { container } = render(<Cockpit evaluation={evaluation} error={null} />);

    expect(getRow(container, "desert")).toHaveTextContent("—");
    expect(getRow(container, "overall")).toHaveTextContent("—");
  });

  it("blanks the ticket value once no pyramid ticket is available", () => {
    const { container } = render(
      <Cockpit evaluation={evaluation} error={null} diceAvailable={false} />,
    );
    const row = getRow(container, "dice");

    expect(row).toHaveTextContent("—");
    expect(row).not.toHaveTextContent("1.00");
  });

  it("replaces leg-bet values with a placeholder when the API marks them unavailable", () => {
    const setup: CockpitResponse = {
      ...evaluation,
      dice: { expectedCoins: null },
      legBet: { ...evaluation.legBet, available: false },
    };
    const { container } = render(<Cockpit evaluation={setup} error={null} />);
    const row = getRow(container, "legBet");

    expect(getRow(container, "dice")).toHaveTextContent("—");
    expect(row).toHaveAttribute("data-leg-bet-available", "false");
    expect(row).toHaveTextContent("Take a leg-betting tile");
    expect(row).toHaveTextContent("—");
    expect(row).not.toHaveTextContent("2.35");
    expect(row?.querySelector("[data-cockpit-camel]")).not.toBeInTheDocument();
  });

  it("keeps the leg-bet values when taking a tile is available", () => {
    const { container } = render(<Cockpit evaluation={evaluation} error={null} />);
    const row = getRow(container, "legBet");

    expect(row).toHaveAttribute("data-leg-bet-available", "true");
    expect(row).toHaveTextContent("2.35");
    expect(row?.querySelectorAll("[data-cockpit-camel]")).toHaveLength(5);
  });

  it("is read only", () => {
    const { container } = render(<Cockpit evaluation={evaluation} error={null} />);

    expect(container.querySelectorAll("button, input, select, textarea")).toHaveLength(0);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("shows placeholders while no evaluation is available", () => {
    const { container } = render(<Cockpit evaluation={null} error={null} />);

    expect(getRow(container, "dice")?.querySelector("[data-expected-coins]")).toHaveTextContent(
      "—",
    );
    expect(getRow(container, "legBet")).toHaveAttribute("data-leg-bet-available", "false");
    expect(container.querySelectorAll("[data-cockpit-camel]")).toHaveLength(0);
    expect(container.querySelector("[data-cockpit-error]")).not.toBeInTheDocument();
  });

  it("shows an explicit backend error while keeping the placeholders", () => {
    const error =
      "Cockpit evaluation failed: cannot reach the Rust backend at http://127.0.0.1:3000.";
    const { container } = render(<Cockpit evaluation={null} error={error} />);

    expect(screen.getByRole("alert")).toHaveTextContent(error);
    expect(container.querySelector("[data-cockpit-error]")).toHaveTextContent(error);
    expect(getRow(container, "legBet")).toHaveTextContent("—");
    expect(container.querySelectorAll("[data-cockpit-camel]")).toHaveLength(0);
  });
});
