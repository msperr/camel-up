import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

// App fetches the cockpit evaluation on mount. The evaluation itself is tested
// in Rust; UI tests stub the transport with a response that mirrors the
// backend's availability rules so they can focus on board behaviour.
vi.stubGlobal(
  "fetch",
  vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
    const body = JSON.parse(String(init?.body ?? "{}")) as {
      setupRound?: boolean;
      thrownDice?: readonly string[];
      finished?: readonly string[] | null;
    };
    const available =
      !body.setupRound && (body.thrownDice?.length ?? 0) < 5 && (body.finished ?? null) === null;

    return {
      ok: true,
      status: 200,
      json: async () => ({
        dice: { expectedCoins: available ? 1 : null },
        legBet: { available, camels: [] },
      }),
    } as Response;
  }),
);

afterEach(() => {
  cleanup();
});
