import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import Cockpit from "./Cockpit";
import type { CockpitResponse } from "./cockpit";

const evaluation: CockpitResponse = {
  dice: { expectedCoins: 1 },
  legBet: {
    camels: [
      { color: "orange", expectedCoins: 2.345, roundWinProbability: 0.425 },
      { color: "white", expectedCoins: 1.2, roundWinProbability: 0.2 },
      { color: "blue", expectedCoins: null, roundWinProbability: 0.31 },
    ],
  },
};

const getRow = (container: HTMLElement, action: string) =>
  container.querySelector<HTMLElement>(`[data-action="${action}"]`);

describe("Cockpit", () => {
  it("shows the four rules actions in order", () => {
    const { container } = render(<Cockpit evaluation={evaluation} />);
    const rows = [...container.querySelectorAll<HTMLElement>("[data-action]")];

    expect(rows.map((row) => row.dataset.action)).toEqual(["dice", "desert", "legBet", "overall"]);
    expect(getRow(container, "dice")).toHaveTextContent("Take a pyramid ticket and roll");
    expect(getRow(container, "desert")).toHaveTextContent("Place or move your desert tile");
    expect(getRow(container, "legBet")).toHaveTextContent("Take a leg-betting tile");
    expect(getRow(container, "overall")).toHaveTextContent("Bet on the overall winner or loser");
  });

  it("shows the guaranteed coin win for rolling without camel columns", () => {
    const { container } = render(<Cockpit evaluation={evaluation} />);
    const row = getRow(container, "dice");

    expect(row).toHaveTextContent("1.00");
    expect(row?.querySelector("[data-cockpit-camel]")).not.toBeInTheDocument();
    expect(
      getRow(container, "desert")?.querySelector("[data-cockpit-camel]"),
    ).not.toBeInTheDocument();
  });

  it("renders two formatted values per camel in the order supplied by the API", () => {
    const { container } = render(<Cockpit evaluation={evaluation} />);
    const camels = [...container.querySelectorAll<HTMLElement>("[data-cockpit-camel]")];

    expect(camels.map((camel) => camel.dataset.cockpitCamel)).toEqual(["orange", "white", "blue"]);
    expect(camels[0].querySelector("[data-expected-coins]")).toHaveTextContent("2.35");
    expect(camels[0].querySelector("[data-win-probability]")).toHaveTextContent("43%");
    expect(camels[1].querySelector("[data-expected-coins]")).toHaveTextContent("1.20");
    expect(camels[1].querySelector("[data-win-probability]")).toHaveTextContent("20%");
  });

  it("leaves coins blank but shows the win probability for an exhausted stack", () => {
    const { container } = render(<Cockpit evaluation={evaluation} />);
    const blue = container.querySelector<HTMLElement>('[data-cockpit-camel="blue"]');

    expect(blue?.querySelector("[data-expected-coins]")).toBeEmptyDOMElement();
    expect(blue?.querySelector("[data-win-probability]")).toHaveTextContent("31%");
  });

  it("uses dashes for the unimplemented actions", () => {
    const { container } = render(<Cockpit evaluation={evaluation} />);

    expect(getRow(container, "desert")).toHaveTextContent("—");
    expect(getRow(container, "overall")).toHaveTextContent("—");
  });

  it("is read only", () => {
    const { container } = render(<Cockpit evaluation={evaluation} />);

    expect(container.querySelectorAll("button, input, select, textarea")).toHaveLength(0);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("shows blank values while no evaluation is available", () => {
    const { container } = render(<Cockpit evaluation={null} />);

    expect(getRow(container, "dice")?.querySelector("[data-expected-coins]")).toBeEmptyDOMElement();
    expect(container.querySelector("[data-cockpit-camel]")).not.toBeInTheDocument();
  });
});
