import { CAMEL_COLORS } from "./camelRace";
import type { CamelOdds, CockpitResponse } from "./cockpit";

type CockpitProps = {
  readonly evaluation: CockpitResponse | null;
  readonly error: string | null;
  /** Whether a pyramid ticket is still available to take this leg. */
  readonly diceAvailable?: boolean;
  /** Whether a leg-betting tile can be taken (false throughout setup). */
  readonly legBetAvailable?: boolean;
};

const formatCoins = (coins: number | null): string => (coins === null ? "" : coins.toFixed(2));

const formatProbability = (probability: number | null): string =>
  probability === null ? "" : `${Math.round(probability * 100)}%`;

const blankCamel = (color: (typeof CAMEL_COLORS)[number]): CamelOdds => ({
  color,
  expectedCoins: null,
  roundWinProbability: null,
});

const camelsForEvaluation = (evaluation: CockpitResponse | null): readonly CamelOdds[] => {
  const fromApi = evaluation?.legBet.camels ?? [];
  const present = new Set(fromApi.map(({ color }) => color));
  const missing = CAMEL_COLORS.filter((color) => !present.has(color)).map(blankCamel);

  return [...fromApi, ...missing];
};

/**
 * Read-only action overview below the board.
 *
 * Row labels are the action names from RULES.md sections 5 to 8. Only the
 * pyramid-ticket roll and the leg-betting tile are evaluated; the desert and
 * overall-bet rows are placeholders. The leg-betting values come from the Rust
 * backend, so an unavailable backend is reported here instead of silently
 * removing the camel columns. The ticket value is blanked once the leg is over
 * and no ticket can be taken. Leg-bet values are replaced by a placeholder
 * throughout the setup round, when taking a tile is not an available action.
 */
export default function Cockpit({
  evaluation,
  error,
  diceAvailable = true,
  legBetAvailable = true,
}: CockpitProps) {
  const camels = camelsForEvaluation(evaluation);

  return (
    <section className="cockpit" aria-label="Actions" data-cockpit="true">
      {error && (
        <p className="cockpit-error" role="alert" data-cockpit-error>
          {error}
        </p>
      )}

      <div className="cockpit-row" data-action="dice">
        <span className="cockpit-label">Take a pyramid ticket and roll</span>
        <span
          className={diceAvailable ? "cockpit-value" : "cockpit-placeholder"}
          data-expected-coins
        >
          {diceAvailable ? formatCoins(evaluation?.dice.expectedCoins ?? null) : "—"}
        </span>
      </div>

      <div className="cockpit-row" data-action="desert">
        <span className="cockpit-label">Place or move your desert tile</span>
        <span className="cockpit-placeholder">—</span>
      </div>

      <div className="cockpit-row" data-action="legBet" data-leg-bet-available={legBetAvailable}>
        <span className="cockpit-label">Take a leg-betting tile</span>
        {legBetAvailable ? (
          <div className="cockpit-camels">
            {camels.map(({ color, expectedCoins, roundWinProbability }) => (
              <div
                className={`cockpit-camel cockpit-camel--${color}`}
                data-cockpit-camel={color}
                key={color}
              >
                <span className="cockpit-swatch" aria-hidden="true" />
                <span className="cockpit-coins" data-expected-coins>
                  {formatCoins(expectedCoins)}
                </span>
                <span className="cockpit-probability" data-win-probability>
                  {formatProbability(roundWinProbability)}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <span className="cockpit-placeholder">—</span>
        )}
      </div>

      <div className="cockpit-row" data-action="overall">
        <span className="cockpit-label">Bet on the overall winner or loser</span>
        <span className="cockpit-placeholder">—</span>
      </div>
    </section>
  );
}
