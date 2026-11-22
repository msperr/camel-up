//! Evaluation data for the read-only player cockpit.
//!
//! RULES.md defines four actions (sections 5 to 8). Only two are evaluated
//! here: taking a pyramid ticket and rolling, whose ticket is worth a
//! guaranteed +1 EP at the next leg scoring (section 7), and taking the top
//! leg-betting tile of a colour, whose net expected EP is
//! `(v + 1) * p1 + 2 * p2 - 1` (section 5). `p1` is the probability that the
//! camel is 1st at the end of the current leg; `p2` is the probability that it
//! is 2nd. The other two actions are presented as placeholders in the UI.
//!
//! When every camel is on the board, `p1`/`p2` come from the exhaustive
//! [`State::simulate_outcomes`]. The temporary frontend board state can also
//! contain camels that have not been placed yet; for those states a fixed-seed
//! Monte Carlo simulation supplies the probabilities, so the cockpit works
//! from the first die onwards. A completely empty board has no evaluable
//! probabilities and every camel value is `null` (rendered as a blank field).

use crate::simulation::{CAMEL_ORDER, DEFAULT_SEED};
use crate::{Camel, Space, State, MAX_FIELD};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Guaranteed value of a pyramid ticket at the next leg scoring (RULES.md §7).
pub const PYRAMID_TICKET_EP: f64 = 1.0;

/// Scenarios used for boards where some camels have not been placed yet.
const PARTIAL_BOARD_ITERATIONS: usize = 10_000;

/// JSON colour name, kept separate from the rules model so the core types do
/// not depend on serde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CamelColor {
    White,
    Yellow,
    Orange,
    Green,
    Blue,
}

impl CamelColor {
    fn rules_camel(self) -> Camel {
        match self {
            CamelColor::White => Camel::White,
            CamelColor::Yellow => Camel::Yellow,
            CamelColor::Orange => Camel::Orange,
            CamelColor::Green => Camel::Green,
            CamelColor::Blue => Camel::Blue,
        }
    }

    fn from_rules(camel: Camel) -> Self {
        match camel {
            Camel::White => CamelColor::White,
            Camel::Yellow => CamelColor::Yellow,
            Camel::Orange => CamelColor::Orange,
            Camel::Green => CamelColor::Green,
            Camel::Blue => CamelColor::Blue,
        }
    }
}

/// Board snapshot sent by the cockpit. Space keys are 1..=16; a finished camel
/// unit is sent separately because it is stored off the track by `State`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CockpitRequest {
    #[serde(default)]
    pub stacks: BTreeMap<u8, Vec<CamelColor>>,
    #[serde(default)]
    pub finished: Option<Vec<CamelColor>>,
    #[serde(default)]
    pub thrown_dice: Vec<CamelColor>,
    /// Printed value of the top available leg-betting tile, `null` when the
    /// colour's stack is exhausted.
    #[serde(default)]
    pub available_leg_bets: BTreeMap<CamelColor, Option<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CockpitResponse {
    pub dice: DiceAction,
    pub leg_bet: LegBetAction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiceAction {
    pub expected_coins: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegBetAction {
    /// Already sorted by expected coins, then `p1`, then colour; exhausted
    /// stacks come last.
    pub camels: Vec<CamelOdds>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CamelOdds {
    pub color: CamelColor,
    /// Net expected EP of taking this colour's top leg-betting tile.
    pub expected_coins: Option<f64>,
    /// `p1`: probability of being 1st at the end of the current leg. Present
    /// even when the colour has no leg-betting tile left; `None` only when the
    /// board has no evaluable outcomes.
    pub round_win_probability: Option<f64>,
}

/// Evaluate both implemented actions. `Err` contains a validation message for
/// malformed snapshots, which the HTTP layer turns into a 400 response.
pub fn evaluate_cockpit(request: &CockpitRequest) -> Result<CockpitResponse, String> {
    let state = build_state(request)?;
    let already_moved: Vec<Camel> = request
        .thrown_dice
        .iter()
        .map(|&color| color.rules_camel())
        .collect();
    let available: BTreeMap<Camel, Option<u8>> = request
        .available_leg_bets
        .iter()
        .map(|(&color, &value)| (color.rules_camel(), value))
        .collect();

    let counts = if state.data.is_empty() {
        BTreeMap::new()
    } else if state.is_finished() || already_moved.len() == CAMEL_ORDER.len() {
        deterministic_counts(&state)
    } else if unplaced_camels(&state).is_empty() {
        state.simulate_outcomes(&already_moved).0
    } else {
        let unplaced = unplaced_camels(&state);
        simulate_partial_board(&state, &already_moved, &unplaced)
    };

    let total: usize = counts
        .values()
        .next()
        .map(|positions| positions.iter().sum())
        .unwrap_or(0);

    let mut camels: Vec<CamelOdds> = CAMEL_ORDER
        .iter()
        .map(|&camel| {
            if total == 0 {
                return CamelOdds {
                    color: CamelColor::from_rules(camel),
                    expected_coins: None,
                    round_win_probability: None,
                };
            }

            let positions = &counts[&camel];
            let p1 = positions[0] as f64 / total as f64;
            let p2 = positions[1] as f64 / total as f64;
            let expected_coins = available
                .get(&camel)
                .copied()
                .flatten()
                .map(|value| (value as f64 + 1.0) * p1 + 2.0 * p2 - 1.0);
            CamelOdds {
                color: CamelColor::from_rules(camel),
                expected_coins,
                round_win_probability: Some(p1),
            }
        })
        .collect();

    camels.sort_by(compare_camel_odds);

    Ok(CockpitResponse {
        dice: DiceAction {
            expected_coins: PYRAMID_TICKET_EP,
        },
        leg_bet: LegBetAction { camels },
    })
}

fn compare_camel_odds(left: &CamelOdds, right: &CamelOdds) -> Ordering {
    match (left.expected_coins, right.expected_coins) {
        (Some(left_ev), Some(right_ev)) => right_ev
            .total_cmp(&left_ev)
            .then_with(|| compare_probability_then_color(left, right)),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => compare_probability_then_color(left, right),
    }
}

fn compare_probability_then_color(left: &CamelOdds, right: &CamelOdds) -> Ordering {
    right
        .round_win_probability
        .unwrap_or(0.0)
        .total_cmp(&left.round_win_probability.unwrap_or(0.0))
        .then_with(|| left.color.cmp(&right.color))
}

fn build_state(request: &CockpitRequest) -> Result<State, String> {
    let mut data: BTreeMap<u8, Space> = BTreeMap::new();
    let mut seen = BTreeMap::new();

    for (&space, colors) in &request.stacks {
        if space == 0 || space > MAX_FIELD {
            return Err(format!("space {} must be in 1..={}", space, MAX_FIELD));
        }
        if colors.is_empty() {
            return Err(format!("space {} has no camels", space));
        }
        for &color in colors {
            register_camel(&mut seen, color.rules_camel())?;
        }
        data.insert(
            space,
            Space::Camels(colors.iter().map(|&color| color.rules_camel()).collect()),
        );
    }

    if let Some(finished) = &request.finished {
        if finished.is_empty() {
            return Err("the finished camel unit must not be empty".to_string());
        }
        for &color in finished {
            register_camel(&mut seen, color.rules_camel())?;
        }
        data.insert(
            MAX_FIELD + 1,
            Space::Camels(finished.iter().map(|&color| color.rules_camel()).collect()),
        );
    }

    if seen.len() > CAMEL_ORDER.len() {
        return Err("the board cannot contain more than five camels".to_string());
    }

    let mut thrown = BTreeMap::new();
    for &color in &request.thrown_dice {
        if thrown.insert(color, ()).is_some() {
            return Err(format!("die for {:?} was thrown twice", color));
        }
    }
    if request.thrown_dice.len() > CAMEL_ORDER.len() {
        return Err("more than five dice were thrown".to_string());
    }

    for (&color, &value) in &request.available_leg_bets {
        if let Some(value) = value {
            if !matches!(value, 5 | 3 | 2) {
                return Err(format!(
                    "invalid leg-betting tile value {} for {:?}",
                    value, color
                ));
            }
        }
    }

    Ok(State::new(data))
}

fn register_camel(seen: &mut BTreeMap<Camel, ()>, camel: Camel) -> Result<(), String> {
    if seen.insert(camel, ()).is_some() {
        return Err(format!("camel {:?} appears more than once", camel));
    }
    Ok(())
}

/// Camel positions are final: each camel is assigned its place with certainty,
/// as in `expected_leg_bet_coins` when the leg is over.
fn deterministic_counts(state: &State) -> BTreeMap<Camel, Vec<usize>> {
    let mut counts: BTreeMap<Camel, Vec<usize>> = CAMEL_ORDER
        .iter()
        .map(|&camel| (camel, vec![0usize; CAMEL_ORDER.len()]))
        .collect();

    for (position, &camel) in state.ranking().iter().enumerate() {
        counts
            .entry(camel)
            .or_insert_with(|| vec![0usize; CAMEL_ORDER.len()])[position] += 1;
    }
    counts
}

fn unplaced_camels(state: &State) -> Vec<Camel> {
    CAMEL_ORDER
        .iter()
        .copied()
        .filter(|camel| {
            !state.data.values().any(|space| match space {
                Space::Camels(stack) => stack.contains(camel),
                Space::Desert(_) => false,
            })
        })
        .collect()
}

/// The frontend's temporary board can omit camels whose die has not been
/// revealed yet. Each scenario first gives those camels a random starting
/// space (RULES.md §3), then plays the dice still in the pyramid. A fixed seed
/// keeps repeated requests for the same snapshot identical.
fn simulate_partial_board(
    state: &State,
    already_moved: &[Camel],
    unplaced: &[Camel],
) -> BTreeMap<Camel, Vec<usize>> {
    let mut rng = Rng::new(DEFAULT_SEED);
    let mut counts: BTreeMap<Camel, Vec<usize>> = CAMEL_ORDER
        .iter()
        .map(|&camel| (camel, vec![0usize; CAMEL_ORDER.len()]))
        .collect();

    for _ in 0..PARTIAL_BOARD_ITERATIONS {
        let mut scenario = state.clone();
        for &camel in unplaced {
            let value = (rng.below(3) + 1) as u8;
            // Key 0 is a temporary staging space; the camel leaves it on the
            // very next move, so it never affects the ranking.
            scenario.data.insert(0, Space::Camels(vec![camel]));
            scenario = scenario.move_unit(camel, value).0;
        }

        let mut remaining: Vec<Camel> = CAMEL_ORDER
            .iter()
            .copied()
            .filter(|camel| !already_moved.contains(camel))
            .collect();
        for index in (1..remaining.len()).rev() {
            let other = rng.below(index + 1);
            remaining.swap(index, other);
        }
        let moves = remaining
            .into_iter()
            .map(|camel| (camel, (rng.below(3) + 1) as u8));

        let end = scenario.move_multiple_units(moves).0;
        for (position, &camel) in end.ranking().iter().enumerate() {
            counts.get_mut(&camel).expect("all camels are tracked")[position] += 1;
        }
    }

    counts
}

/// Small, self-contained xorshift64* generator, mirroring `simulation::SimRng`.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng {
            state: if seed == 0 { DEFAULT_SEED } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        ((self.next_u64() as u128 * upper as u128) >> 64) as usize
    }
}
