import DesertArtwork from "./DesertArtwork";
import { CAMEL_COLORS } from "./camelRace";
import type { CamelOdds, CockpitResponse, DesertOption } from "./cockpit";

type CockpitProps = {
  readonly evaluation: CockpitResponse | null;
  readonly error: string | null;
  /** Whether a pyramid ticket is still available to take this leg. */
  readonly diceAvailable?: boolean;
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
 * Row labels are the action names from RULES.md sections 5 to 8. The values
 * come from the Rust backend, so an unavailable backend is reported here
 * instead of showing stale values. The backend marks the ticket, leg-bet and
 * desert actions unavailable during setup, after five dice, and at race
 * finish; their values are then replaced by a placeholder. With no evaluation
 * (for example while loading or on error) the placeholders are shown.
 */
export default function Cockpit({ evaluation, error, diceAvailable = true }: CockpitProps) {
  const camels = camelsForEvaluation(evaluation);
  const diceCoins = evaluation?.dice.expectedCoins ?? null;
  const legBetAvailable = evaluation?.legBet.available ?? false;
  const desertOptions: readonly DesertOption[] = (evaluation?.desert?.options ?? []).slice(0, 4);

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
          className={diceAvailable && diceCoins !== null ? "cockpit-value" : "cockpit-placeholder"}
          data-expected-coins
        >
          {diceAvailable && diceCoins !== null ? formatCoins(diceCoins) : "—"}
        </span>
      </div>

      <div className="cockpit-row" data-action="desert">
        <span className="cockpit-label">Place or move your desert tile</span>
        {desertOptions.length > 0 ? (
          <div className="cockpit-deserts">
            {desertOptions.map(({ kind, space, expectedCoins }) => (
              <div
                className={`cockpit-desert cockpit-desert--${kind}`}
                data-desert-option="true"
                data-desert-kind={kind}
                data-desert-space={space}
                aria-label={`${kind} on space ${space}`}
                key={`${kind}-${space}`}
              >
                <svg className="cockpit-desert-icon" viewBox="0 0 100 100" aria-hidden="true">
                  <rect width="100" height="100" rx="10" className="desert-tile-surface" />
                  <DesertArtwork kind={kind} />
                </svg>
                <span className="cockpit-desert-space">{space}</span>
                <span className="cockpit-coins" data-expected-coins>
                  {formatCoins(expectedCoins)}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <span className="cockpit-placeholder">—</span>
        )}
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
