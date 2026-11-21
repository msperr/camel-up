//! Monte Carlo simulation of the overall winner and loser camel.
//!
//! Unlike [`crate::State::simulate_outcomes`], which enumerates every camel
//! permutation and die combination of a single leg, this module repeatedly
//! plays random scenarios from the current state until a camel crosses the
//! finish line. Each scenario contributes one overall winner and one overall
//! loser, as used by the end-of-game race bets (RULES.md sections 8 and 9).

use crate::{Camel, Space, State};

/// Colour order used for the ten returned shares.
pub const CAMEL_ORDER: [Camel; 5] = [
    Camel::White,
    Camel::Yellow,
    Camel::Orange,
    Camel::Green,
    Camel::Blue,
];

/// Default seed of the built-in pseudo-random generator.
pub const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// Number of random scenarios played per call to
/// [`simulate_overall_winner_loser`]. Measured with the calibration harness in
/// `tests/simulation.rs` against a 1,000,000-scenario reference: 10,000
/// scenarios give a maximum observed error of 0.66 percentage points and take
/// about 0.3 s in a release build (about 2.4 s in debug). Larger sample sizes
/// cost disproportionately more time for little precision gain.
pub static SIMULATION_ITERATIONS: usize = 10_000;

/// Ten probabilities: `result[0..5]` are the shares of being the overall
/// winner, `result[5..10]` the shares of being the overall loser, both in
/// [`CAMEL_ORDER`]. Each half sums to 1.0, so the whole array sums to 2.0.
pub type OverallOdds = [f64; 10];

/// Small, self-contained xorshift64* generator. A fixed seed makes results
/// reproducible and avoids adding an external dependency to the crate.
struct SimRng {
    state: u64,
}

impl SimRng {
    fn new(seed: u64) -> Self {
        SimRng {
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

    /// Uniform integer in `0..upper` (upper <= 5 in this module).
    fn below(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        // Multiply-high avoids the small modulo bias of `% upper`.
        ((self.next_u64() as u128 * upper as u128) >> 64) as usize
    }
}

fn camel_index(camel: Camel) -> usize {
    CAMEL_ORDER
        .iter()
        .position(|&c| c == camel)
        .expect("camel must be one of the five racing camels")
}

/// Whether a camel unit has crossed the finish line (space 16).
fn is_finished(state: &State) -> bool {
    state.is_finished()
}

/// Return the winner (top of the leading stack) and loser (bottom of the
/// rearmost stack) of an already finished state. See RULES.md section 2.
fn finished_camels(state: &State) -> (Camel, Camel) {
    let ranking = state.ranking(); // 1st place -> last place
    let winner = *ranking
        .first()
        .expect("a finished state must contain at least one camel");
    let loser = *ranking
        .last()
        .expect("a finished state must contain at least one camel");
    (winner, loser)
}

/// Desert tiles are returned to their owners at the end of a leg (RULES.md
/// section 9). This simulation models no player actions, so no new tiles are
/// placed: remove the ones present in the starting state when the leg ends.
fn clear_deserts(state: &mut State) {
    state.data.retain(|_, space| match space {
        Space::Camels(_) => true,
        Space::Desert(_) => false,
    });
}

fn validate(state: &State, already_moved: &[Camel], iterations: usize) {
    assert!(iterations > 0, "iterations must be greater than zero");

    for (i, &camel) in already_moved.iter().enumerate() {
        assert!(
            !already_moved[..i].contains(&camel),
            "camel {:?} appears multiple times in already_moved",
            camel
        );
    }

    for &camel in &CAMEL_ORDER {
        let occurrences: usize = state
            .data
            .values()
            .map(|space| match space {
                Space::Camels(v) => v.iter().filter(|&&c| c == camel).count(),
                Space::Desert(_) => 0usize,
            })
            .sum();
        assert_eq!(
            occurrences, 1,
            "simulation requires each Camel to appear exactly once in state.data; camel {:?} appears {} times",
            camel, occurrences
        );
    }
}

/// Play one random scenario from `state` and return its overall winner/loser.
fn run_scenario(state: &State, rng: &mut SimRng, already_moved: &[Camel]) -> (Camel, Camel) {
    let mut current = state.clone();
    let mut remaining: Vec<Camel> = CAMEL_ORDER
        .iter()
        .copied()
        .filter(|camel| !already_moved.contains(camel))
        .collect();

    loop {
        if remaining.is_empty() {
            // All five dice have been revealed: the leg is over. The dice go
            // back into the pyramid and desert tiles are returned to owners.
            clear_deserts(&mut current);
            remaining = CAMEL_ORDER.to_vec();
        }

        // RULES.md section 7: reveal one random die from the pyramid, so the
        // colour is drawn without replacement within a leg and its value is
        // uniformly one of 1, 2, 3 (each appears twice on every die).
        let color_index = rng.below(remaining.len());
        let camel = remaining.swap_remove(color_index);
        let steps = (rng.below(3) + 1) as u8;

        let (next, _) = current.move_unit(camel, steps);
        current = next;

        // move_unit ends the race on a crossing, including an oasis on space 16
        // pushing past the line. Stop the scenario at that point.
        if is_finished(&current) {
            return finished_camels(&current);
        }
    }
}

/// Simulate overall winner/loser shares with explicit parameters.
///
/// `already_moved` lists camels whose dice have already been revealed in the
/// current leg; `self` must already reflect those moves. An empty slice starts
/// a fresh leg with all five dice. Passing all five camels is valid here: the
/// simulation simply starts the next leg (unlike `simulate_outcomes`).
pub fn simulate_overall_winner_loser_with(
    state: &State,
    iterations: usize,
    seed: u64,
    already_moved: &[Camel],
) -> OverallOdds {
    validate(state, already_moved, iterations);

    if is_finished(state) {
        let (winner, loser) = finished_camels(state);
        let mut result = [0.0f64; 10];
        result[camel_index(winner)] = 1.0;
        result[5 + camel_index(loser)] = 1.0;
        return result;
    }

    let mut winner_counts = [0usize; 5];
    let mut loser_counts = [0usize; 5];
    let mut rng = SimRng::new(seed);

    for _ in 0..iterations {
        let (winner, loser) = run_scenario(state, &mut rng, already_moved);
        winner_counts[camel_index(winner)] += 1;
        loser_counts[camel_index(loser)] += 1;
    }

    let denominator = iterations as f64;
    let mut result = [0.0f64; 10];
    for i in 0..5 {
        result[i] = winner_counts[i] as f64 / denominator;
        result[5 + i] = loser_counts[i] as f64 / denominator;
    }
    result
}

/// Simulate [`SIMULATION_ITERATIONS`] scenarios from the current state using
/// the default seed. Desert tiles present in the starting state apply to the
/// first leg only and are not replaced afterwards.
pub fn simulate_overall_winner_loser(state: &State) -> OverallOdds {
    simulate_overall_winner_loser_with(state, SIMULATION_ITERATIONS, DEFAULT_SEED, &[])
}
