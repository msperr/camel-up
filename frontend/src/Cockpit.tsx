import type { CockpitResponse } from "./cockpit";

type CockpitProps = {
  readonly evaluation: CockpitResponse | null;
};

const formatCoins = (coins: number | null): string => (coins === null ? "" : coins.toFixed(2));

const formatProbability = (probability: number | null): string =>
  probability === null ? "" : `${Math.round(probability * 100)}%`;

/**
 * Read-only action overview below the board.
 *
 * Row labels are the action names from RULES.md sections 5 to 8. Only the
 * pyramid-ticket roll and the leg-betting tile are evaluated; the desert and
 * overall-bet rows are placeholders.
 */
export default function Cockpit({ evaluation }: CockpitProps) {
  const camels = evaluation?.legBet.camels ?? [];

  return (
    <section className="cockpit" aria-label="Actions" data-cockpit="true">
      <div className="cockpit-row" data-action="dice">
        <span className="cockpit-label">Take a pyramid ticket and roll</span>
        <span className="cockpit-value" data-expected-coins>
          {formatCoins(evaluation?.dice.expectedCoins ?? null)}
        </span>
      </div>

      <div className="cockpit-row" data-action="desert">
        <span className="cockpit-label">Place or move your desert tile</span>
        <span className="cockpit-placeholder">—</span>
      </div>

      <div className="cockpit-row" data-action="legBet">
        <span className="cockpit-label">Take a leg-betting tile</span>
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
      </div>

      <div className="cockpit-row" data-action="overall">
        <span className="cockpit-label">Bet on the overall winner or loser</span>
        <span className="cockpit-placeholder">—</span>
      </div>
    </section>
  );
}
