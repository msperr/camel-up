import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

// App fetches the cockpit evaluation on mount. The evaluation itself is
// tested in Rust; UI tests stub the transport with an empty result so they can
// focus on board behaviour.
vi.stubGlobal(
  "fetch",
  vi.fn(async () => ({
    ok: true,
    status: 200,
    json: async () => ({ dice: { expectedCoins: 1 }, legBet: { available: true, camels: [] } }),
  })),
);

afterEach(() => {
  cleanup();
});
