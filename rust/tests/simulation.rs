use camel_up::simulation::{
    simulate_overall_winner_loser, simulate_overall_winner_loser_with, CAMEL_ORDER, DEFAULT_SEED,
    SIMULATION_ITERATIONS,
};
use camel_up::{Camel, DesertTile, Space, State};
use std::collections::BTreeMap;
use std::time::Instant;

const ALL: [Camel; 5] = [
    Camel::White,
    Camel::Yellow,
    Camel::Orange,
    Camel::Green,
    Camel::Blue,
];

fn index(camel: Camel) -> usize {
    CAMEL_ORDER.iter().position(|&c| c == camel).unwrap()
}

fn map(entries: &[(u8, Space)]) -> BTreeMap<u8, Space> {
    entries.iter().cloned().collect()
}

fn base_state() -> State {
    State::new(map(&[
        (1, Space::Camels(vec![Camel::White])),
        (2, Space::Camels(vec![Camel::Yellow, Camel::Orange])),
        (4, Space::Camels(vec![Camel::Green])),
        (6, Space::Camels(vec![Camel::Blue])),
        (8, Space::Desert(DesertTile::Oasis)),
    ]))
}

fn assert_valid_shares(shares: &[f64; 10]) {
    let winners: f64 = shares[..5].iter().sum();
    let losers: f64 = shares[5..].iter().sum();
    assert!(
        (winners - 1.0).abs() < 1e-12,
        "winner shares sum to {}",
        winners
    );
    assert!(
        (losers - 1.0).abs() < 1e-12,
        "loser shares sum to {}",
        losers
    );
    assert!((shares.iter().sum::<f64>() - 2.0).abs() < 1e-12);
    assert!(shares.iter().all(|&value| (0.0..=1.0).contains(&value)));
}

#[test]
fn public_entry_point_returns_ten_shares_summing_to_two() {
    let shares = simulate_overall_winner_loser(&base_state());
    assert_eq!(shares.len(), 10);
    assert_valid_shares(&shares);
}

#[test]
fn default_wrapper_uses_static_iterations_and_default_seed() {
    let state = base_state();
    assert_eq!(
        simulate_overall_winner_loser(&state),
        simulate_overall_winner_loser_with(&state, SIMULATION_ITERATIONS, DEFAULT_SEED, &[])
    );
}

#[test]
fn same_seed_is_reproducible() {
    let state = base_state();
    let first = simulate_overall_winner_loser_with(&state, 2_000, 42, &[]);
    let second = simulate_overall_winner_loser_with(&state, 2_000, 42, &[]);
    assert_eq!(first, second);
}

#[test]
fn camel_at_finish_line_always_wins() {
    let state = State::new(map(&[
        (16, Space::Camels(vec![Camel::White])),
        (2, Space::Camels(vec![Camel::Yellow, Camel::Orange])),
        (3, Space::Camels(vec![Camel::Green, Camel::Blue])),
    ]));

    let shares = simulate_overall_winner_loser_with(&state, 2_000, 7, &[]);
    assert_valid_shares(&shares);
    assert!(
        shares[index(Camel::White)] > 0.99,
        "White should cross first: {:?}",
        shares
    );
}

#[test]
fn rearmost_camel_is_overall_loser() {
    let state = State::new(map(&[
        (1, Space::Camels(vec![Camel::White])),
        (
            14,
            Space::Camels(vec![
                Camel::Yellow,
                Camel::Orange,
                Camel::Green,
                Camel::Blue,
            ]),
        ),
    ]));

    let shares = simulate_overall_winner_loser_with(&state, 2_000, 9, &[]);
    assert_valid_shares(&shares);
    assert!(
        shares[5 + index(Camel::White)] > 0.99,
        "White should be last: {:?}",
        shares
    );
}

#[test]
fn desert_tiles_are_removed_when_the_leg_ends() {
    let with_desert = State::new(map(&[
        (1, Space::Camels(vec![Camel::White])),
        (2, Space::Camels(vec![Camel::Yellow])),
        (3, Space::Camels(vec![Camel::Orange])),
        (4, Space::Camels(vec![Camel::Green])),
        (5, Space::Camels(vec![Camel::Blue])),
        (9, Space::Desert(DesertTile::Oasis)),
    ]));
    let without_desert = State::new(map(&[
        (1, Space::Camels(vec![Camel::White])),
        (2, Space::Camels(vec![Camel::Yellow])),
        (3, Space::Camels(vec![Camel::Orange])),
        (4, Space::Camels(vec![Camel::Green])),
        (5, Space::Camels(vec![Camel::Blue])),
    ]));

    // All five dice are already gone, so the simulation starts by returning
    // the desert tile to its owner. Both states must then behave identically.
    let with = simulate_overall_winner_loser_with(&with_desert, 2_000, 77, &ALL);
    let without = simulate_overall_winner_loser_with(&without_desert, 2_000, 77, &ALL);
    assert_eq!(with, without);
}

#[test]
fn oasis_on_space_16_causes_a_finish() {
    let state = State::new(map(&[
        (15, Space::Camels(vec![Camel::White])),
        (1, Space::Camels(vec![Camel::Yellow])),
        (2, Space::Camels(vec![Camel::Orange])),
        (3, Space::Camels(vec![Camel::Green])),
        (4, Space::Camels(vec![Camel::Blue])),
        (16, Space::Desert(DesertTile::Oasis)),
    ]));

    let shares = simulate_overall_winner_loser_with(&state, 2_000, 5, &[]);
    assert_valid_shares(&shares);
    assert_eq!(shares[index(Camel::White)], 1.0);
}

#[test]
fn lower_camel_carries_stack_over_finish_and_top_camel_wins() {
    let state = State::new(map(&[
        (15, Space::Camels(vec![Camel::White, Camel::Yellow])),
        (1, Space::Camels(vec![Camel::Orange])),
        (2, Space::Camels(vec![Camel::Green])),
        (3, Space::Camels(vec![Camel::Blue])),
    ]));

    // Yellow's die was already revealed, so White always moves the whole
    // stack and Yellow cannot leave it before the finish.
    let shares = simulate_overall_winner_loser_with(&state, 2_000, 6, &[Camel::Yellow]);
    assert_valid_shares(&shares);
    assert_eq!(shares[index(Camel::Yellow)], 1.0);
}

#[test]
fn balanced_board_gives_every_camel_a_nonzero_share() {
    let state = State::new(map(&[
        (1, Space::Camels(vec![Camel::White])),
        (3, Space::Camels(vec![Camel::Yellow])),
        (5, Space::Camels(vec![Camel::Orange])),
        (7, Space::Camels(vec![Camel::Green])),
        (9, Space::Camels(vec![Camel::Blue])),
    ]));

    let shares = simulate_overall_winner_loser(&state);
    assert_valid_shares(&shares);
    assert!(
        shares.iter().all(|&share| share > 0.0),
        "every camel should get a non-zero share: {:?}",
        shares
    );
}

#[test]
fn finished_state_needs_no_rolls() {
    let state = State::new(map(&[
        (17, Space::Camels(vec![Camel::White])),
        (2, Space::Camels(vec![Camel::Yellow, Camel::Orange])),
        (3, Space::Camels(vec![Camel::Green])),
        (4, Space::Camels(vec![Camel::Blue])),
    ]));

    let shares = simulate_overall_winner_loser_with(&state, 100, 1, &[]);
    assert_valid_shares(&shares);
    assert_eq!(shares[index(Camel::White)], 1.0);
    assert_eq!(shares[5 + index(Camel::Yellow)], 1.0);
}

#[test]
fn all_camels_already_moved_starts_a_new_leg() {
    let shares = simulate_overall_winner_loser_with(&base_state(), 2_000, 123, &ALL);
    assert_valid_shares(&shares);
}

#[test]
fn partial_leg_is_accepted() {
    let shares =
        simulate_overall_winner_loser_with(&base_state(), 2_000, 123, &[Camel::White, Camel::Blue]);
    assert_valid_shares(&shares);
}

#[test]
#[should_panic(expected = "appears multiple times in already_moved")]
fn duplicate_already_moved_panics() {
    simulate_overall_winner_loser_with(
        &base_state(),
        10,
        1,
        &[Camel::White, Camel::Orange, Camel::White],
    );
}

#[test]
#[should_panic(expected = "camel Blue appears 0 times")]
fn missing_camel_panics() {
    let mut data = base_state().data;
    data.remove(&6);
    simulate_overall_winner_loser_with(&State::new(data), 10, 1, &[]);
}

#[test]
#[should_panic(expected = "iterations must be greater than zero")]
fn zero_iterations_panics() {
    simulate_overall_winner_loser_with(&base_state(), 0, 1, &[]);
}

/// Calibration harness: run with
/// `cargo test --test simulation calibrate_iterations -- --ignored --nocapture`
#[test]
#[ignore = "calibration harness; run explicitly with --ignored --nocapture"]
fn calibrate_iterations() {
    let reference_iterations = 1_000_000usize;
    let reference =
        simulate_overall_winner_loser_with(&base_state(), reference_iterations, 0xC0FFEE, &[]);
    let seeds = [DEFAULT_SEED, 11, 22, 33];

    println!(
        "reference: {} scenarios, seed 0x{:X}",
        reference_iterations, 0xC0FFEEu64
    );
    println!(
        "{:>10} {:>12} {:>12} {:>12}",
        "scenarios", "max_error", "worst_se", "time_ms"
    );

    for &iterations in &[1_000usize, 10_000, 50_000, 100_000] {
        let start = Instant::now();
        let mut max_error = 0.0f64;
        for &seed in &seeds {
            let estimate = simulate_overall_winner_loser_with(&base_state(), iterations, seed, &[]);
            for i in 0..10 {
                max_error = max_error.max((estimate[i] - reference[i]).abs());
            }
        }
        let elapsed_ms = start.elapsed().as_millis();
        // Worst-case standard error for a share (p = 0.5).
        let worst_se = 0.5 / (iterations as f64).sqrt();
        println!(
            "{:>10} {:>12.5} {:>12.5} {:>12}",
            iterations, max_error, worst_se, elapsed_ms
        );
    }
}
