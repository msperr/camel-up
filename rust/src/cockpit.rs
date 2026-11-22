//! Evaluation data for the read-only player cockpit.
//!
//! RULES.md defines four actions (sections 5 to 8). Only two are evaluated
//! here: taking a pyramid ticket and rolling, whose ticket is worth a
//! guaranteed +1 EP at the next leg scoring (section 7), and taking the top
//! leg-betting tile of a colour, whose net expected EP is
//! `(v + 1) * p1 + 2 * p2 - 1` (section 5). `p1` is the probability that the
//! camel is 1st at the end of the current leg; `p2` is the probability that it
//! is 2nd. Desert placements are also evaluated, but only for direct income
//! (1 coin per camel unit landing on the tile); their effect on bet values is
//! not modelled. Overall bets remain a placeholder in the UI.
//!
//! When every camel is on the board, `p1`/`p2` come from the exhaustive
//! [`State::simulate_outcomes`]. The temporary frontend board state can also
//! contain camels that have not been placed yet; for those states a fixed-seed
//! Monte Carlo simulation supplies the probabilities. A completely empty board
//! has no evaluable probabilities and every camel value is `null` (rendered as
//! a blank field).
//!
//! During the setup round (RULES.md section 3) the five dice only place the
//! camels and no leg bet or desert tile may be placed. A snapshot marked
//! `setupRound` therefore reports all three actions as unavailable and returns
//! no values for them, even though its partial-board probabilities can still be
//! simulated for other callers.
//!
//! Snapshots must be internally consistent: every camel named in `thrownDice`
//! must be on the board, and before all five camels are placed each camel on
//! the board must have a thrown die. Otherwise the endpoint returns 400.
//!
//! RULES.md section 3 leaves the order of camels sharing a setup space to the
//! players. Because the frontend enters dice one at a time, the partial-board
//! simulation draws that order afresh for each scenario, so no colour gets a
//! systematic advantage from a fixed stacking order.

use crate::simulation::{CAMEL_ORDER, DEFAULT_SEED};
use crate::{Camel, DesertTile, Space, State, MAX_FIELD};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// Guaranteed value of a pyramid ticket at the next leg scoring (RULES.md §7).
pub const PYRAMID_TICKET_EP: f64 = 1.0;

/// Scenarios used for boards where some camels have not been placed yet.
const PARTIAL_BOARD_ITERATIONS: usize = 10_000;

/// Completed-board samples used to estimate desert income when some camels
/// have not been placed yet. Each candidate is simulated once per sample, so a
/// fixed seed keeps the result deterministic and the endpoint responsive.
const DESERT_PARTIAL_SAMPLES: usize = 1_000;

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

/// Desert tile face, matching the JSON `kind` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DesertKind {
    Oasis,
    Mirage,
}

impl DesertKind {
    fn rules_tile(self) -> DesertTile {
        match self {
            DesertKind::Oasis => DesertTile::Oasis,
            DesertKind::Mirage => DesertTile::Mirage,
        }
    }

    fn from_rules(tile: DesertTile) -> Self {
        match tile {
            DesertTile::Oasis => DesertKind::Oasis,
            DesertTile::Mirage => DesertKind::Mirage,
        }
    }
}

/// A desert tile already placed on the cockpit board (RULES.md section 6).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedDesert {
    pub space: u8,
    pub kind: DesertKind,
}

/// A legal placement and its direct expected income for the rest of the leg.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesertOption {
    pub kind: DesertKind,
    pub space: u8,
    pub expected_coins: f64,
}

/// The desert action's recommendations, best first.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesertAction {
    /// Sorted by expected coins, then space, then oasis before mirage.
    pub options: Vec<DesertOption>,
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
    /// Whether the snapshot is still in the setup round, where the dice only
    /// place the camels and taking a leg bet is not an available action.
    #[serde(default)]
    pub setup_round: bool,
    /// Desert tiles already on the board for the current leg. Defaults to an
    /// empty list so older callers keep working.
    #[serde(default)]
    pub deserts: Vec<PlacedDesert>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CockpitResponse {
    pub dice: DiceAction,
    pub desert: DesertAction,
    pub leg_bet: LegBetAction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiceAction {
    /// `None` when taking a pyramid ticket is not an available action: during
    /// setup, after all five dice are thrown, or once the race is finished.
    pub expected_coins: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegBetAction {
    /// Whether taking a leg-betting tile is an available action: `false`
    /// during setup, after all five dice are thrown, and once the race is
    /// finished. Camel values are still listed for final-ranking display; the
    /// cockpit suppresses them while this flag is `false`.
    pub available: bool,
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
    /// even when the colour has no leg-betting tile left or the leg is over;
    /// `None` only when the board has no evaluable outcomes or values are
    /// suppressed during the setup round.
    pub round_win_probability: Option<f64>,
}

/// Evaluate both implemented actions. `Err` contains a validation message for
/// malformed snapshots, which the HTTP layer turns into a 400 response.
pub fn evaluate_cockpit(request: &CockpitRequest) -> Result<CockpitResponse, String> {
    let state = build_state(request)?;
    validate_snapshot(request, &state)?;

    // Both ticket and leg-bet actions are unavailable during setup, after the
    // fifth die of a leg, and once the race is over. Ranking values are still
    // computed after a leg ends so final standings can be displayed.
    let actions_available = !request.setup_round
        && !state.is_finished()
        && request.thrown_dice.len() < CAMEL_ORDER.len();

    // Setup dice only place the camels: there is no leg to score yet, so emit
    // neither a ticket value nor any leg-bet value.
    if request.setup_round {
        return Ok(CockpitResponse {
            dice: DiceAction {
                expected_coins: None,
            },
            // Desert tiles are placed during play, not in the setup round.
            desert: DesertAction {
                options: Vec::new(),
            },
            leg_bet: LegBetAction {
                available: false,
                camels: CAMEL_ORDER
                    .iter()
                    .map(|&camel| CamelOdds {
                        color: CamelColor::from_rules(camel),
                        expected_coins: None,
                        round_win_probability: None,
                    })
                    .collect(),
            },
        });
    }

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

    let desert = DesertAction {
        options: desert_options(&state, &already_moved),
    };

    Ok(CockpitResponse {
        dice: DiceAction {
            expected_coins: actions_available.then_some(PYRAMID_TICKET_EP),
        },
        leg_bet: LegBetAction {
            available: actions_available,
            camels,
        },
        desert,
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

fn compare_desert_options(left: &DesertOption, right: &DesertOption) -> Ordering {
    right
        .expected_coins
        .total_cmp(&left.expected_coins)
        .then_with(|| left.space.cmp(&right.space))
        .then_with(|| left.kind.cmp(&right.kind))
}

/// Recommended desert placements. There is nothing to place at leg end, after
/// the finish, or before any camel is on the board.
fn desert_options(state: &State, already_moved: &[Camel]) -> Vec<DesertOption> {
    if state.data.is_empty()
        || state.is_finished()
        || already_moved.len() >= CAMEL_ORDER.len()
        || board_camels(state).is_empty()
    {
        return Vec::new();
    }

    let unplaced = unplaced_camels(state);
    let incomes = if unplaced.is_empty() {
        state.expected_desert_income(already_moved)
    } else {
        desert_income_partial_board(state, already_moved, &unplaced)
    };

    let mut options: Vec<DesertOption> = incomes
        .into_iter()
        .flat_map(|(space, per_tile)| {
            per_tile
                .into_iter()
                .map(move |(tile, expected_coins)| DesertOption {
                    kind: DesertKind::from_rules(tile),
                    space,
                    expected_coins,
                })
        })
        .collect();
    options.sort_by(compare_desert_options);
    options
}

/// Estimate desert income when some camels have not been placed yet. Each
/// scenario first completes the board with a fixed-seed draw, then plays one
/// random remainder of the leg with the hypothetical tile in place. Candidates
/// that become illegal in a sample are skipped for that sample.
fn desert_income_partial_board(
    state: &State,
    already_moved: &[Camel],
    unplaced: &[Camel],
) -> BTreeMap<u8, BTreeMap<DesertTile, f64>> {
    let candidates: Vec<(u8, DesertTile)> = state
        .legal_desert_spaces()
        .into_iter()
        .flat_map(|space| [(space, DesertTile::Oasis), (space, DesertTile::Mirage)])
        .collect();
    if candidates.is_empty() {
        return BTreeMap::new();
    }

    let mut rng = Rng::new(DEFAULT_SEED);
    let mut hits: BTreeMap<(u8, DesertTile), usize> = BTreeMap::new();
    let mut totals: BTreeMap<(u8, DesertTile), usize> = BTreeMap::new();

    for _ in 0..DESERT_PARTIAL_SAMPLES {
        let completed = place_unplaced_camels(state, &mut rng, unplaced);
        let legal: BTreeSet<u8> = completed.legal_desert_spaces().into_iter().collect();

        for &(space, tile) in &candidates {
            if !legal.contains(&space) {
                continue;
            }
            let key = (space, tile);
            let mut board = completed.data.clone();
            board.insert(space, Space::Desert(tile));

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

            let hit_counts = State::new(board).simulate_moves_hits(moves);
            if hit_counts.contains_key(&space) {
                *hits.entry(key).or_insert(0) += 1;
            }
            *totals.entry(key).or_insert(0) += 1;
        }
    }

    let mut result: BTreeMap<u8, BTreeMap<DesertTile, f64>> = BTreeMap::new();
    for ((space, tile), total) in totals {
        let hit_count = hits.get(&(space, tile)).copied().unwrap_or(0);
        result
            .entry(space)
            .or_default()
            .insert(tile, hit_count as f64 / total as f64);
    }
    result
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

    let mut placed = BTreeSet::new();
    for desert in &request.deserts {
        if desert.space < 2 || desert.space > MAX_FIELD {
            return Err(format!(
                "desert space {} must be in 2..={}",
                desert.space, MAX_FIELD
            ));
        }
        if !placed.insert(desert.space) {
            return Err(format!("two desert tiles on space {}", desert.space));
        }
        if data.contains_key(&desert.space) {
            return Err(format!("desert space {} is occupied", desert.space));
        }
        for neighbour in [desert.space.checked_sub(1), desert.space.checked_add(1)]
            .into_iter()
            .flatten()
        {
            if matches!(data.get(&neighbour), Some(Space::Desert(_))) {
                return Err(format!(
                    "desert space {} is adjacent to the tile on {}",
                    desert.space, neighbour
                ));
            }
        }
        data.insert(desert.space, Space::Desert(desert.kind.rules_tile()));
    }

    Ok(State::new(data))
}

fn register_camel(seen: &mut BTreeMap<Camel, ()>, camel: Camel) -> Result<(), String> {
    if seen.insert(camel, ()).is_some() {
        return Err(format!("camel {:?} appears more than once", camel));
    }
    Ok(())
}

fn board_camels(state: &State) -> BTreeSet<Camel> {
    let mut camels = BTreeSet::new();
    for space in state.data.values() {
        if let Space::Camels(stack) = space {
            camels.extend(stack.iter().copied());
        }
    }
    camels
}

/// Reject board/dice combinations that cannot occur in the game.
///
/// Two invariants matter here. First, revealing a die always places or moves
/// its camel, so a die cannot be listed for a camel that is absent from the
/// board. Second, while the first leg is still being set up, every camel on
/// the board must have a listed die; once all five camels are placed, later
/// legs start with an empty dice list and any subset of dice may have been
/// revealed.
fn validate_snapshot(request: &CockpitRequest, state: &State) -> Result<(), String> {
    let thrown: BTreeSet<Camel> = request
        .thrown_dice
        .iter()
        .map(|&color| color.rules_camel())
        .collect();
    let on_board = board_camels(state);

    if let Some(&camel) = thrown.difference(&on_board).next() {
        return Err(format!(
            "die for {:?} was thrown but the camel is not on the board",
            camel
        ));
    }

    if on_board.len() < CAMEL_ORDER.len() && thrown.len() != on_board.len() {
        return Err(format!(
            "each of the {} camel(s) on the board needs a thrown die, got {}",
            on_board.len(),
            thrown.len()
        ));
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
/// space (RULES.md §3), then plays the dice still in the pyramid. The order in
/// which camels sharing a space are stacked is also redrawn per scenario,
/// since the rules leave that choice to the players. A fixed seed keeps
/// repeated requests for the same snapshot identical.
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
        let scenario = place_unplaced_camels(state, &mut rng, unplaced);

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

/// Place camels whose setup die has not been revealed yet, drawing both the
/// placement order and their spaces from `rng`. Key 0 is a temporary staging
/// space; a camel leaves it on its very next move.
fn place_unplaced_camels(state: &State, rng: &mut Rng, unplaced: &[Camel]) -> State {
    let mut scenario = state.clone();
    let mut placement_order = unplaced.to_vec();
    for index in (1..placement_order.len()).rev() {
        let other = rng.below(index + 1);
        placement_order.swap(index, other);
    }

    for &camel in &placement_order {
        let value = (rng.below(3) + 1) as u8;
        scenario.data.insert(0, Space::Camels(vec![camel]));
        scenario = scenario.move_unit(camel, value).0;
    }
    scenario
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
