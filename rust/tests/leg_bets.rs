use camel_up::{expected_leg_bet_coins, AvailableLegBets, Camel, Space, State};
use std::collections::BTreeMap;

const ALL: [Camel; 5] = [
    Camel::White,
    Camel::Yellow,
    Camel::Orange,
    Camel::Green,
    Camel::Blue,
];

fn state(entries: &[(u8, Vec<Camel>)]) -> State {
    let mut data = BTreeMap::new();
    for (space, camels) in entries {
        data.insert(*space, Space::Camels(camels.clone()));
    }
    State::new(data)
}

fn top_tiles(values: &[(Camel, u8)]) -> AvailableLegBets {
    values
        .iter()
        .map(|&(camel, value)| (camel, vec![value]))
        .collect()
}

fn assert_close(actual: Option<f64>, expected: f64) {
    assert!(
        (actual.expect("expected a value") - expected).abs() < 1e-12,
        "expected {}, got {:?}",
        expected,
        actual
    );
}

/// One die left: Blue is already ahead of the other camels and always wins.
fn single_die_state() -> State {
    state(&[
        (2, vec![Camel::Green]),
        (3, vec![Camel::Orange]),
        (4, vec![Camel::Yellow]),
        (5, vec![Camel::White]),
        (6, vec![Camel::Blue]),
    ])
}

const FOUR_MOVED: [Camel; 4] = [Camel::White, Camel::Yellow, Camel::Orange, Camel::Green];

#[test]
fn one_die_remaining_uses_first_second_else_payout() {
    let bets = top_tiles(&[
        (Camel::Blue, 3),
        (Camel::White, 5),
        (Camel::Yellow, 3),
        (Camel::Orange, 2),
        (Camel::Green, 5),
    ]);

    let result = expected_leg_bet_coins(&single_die_state(), &bets, &FOUR_MOVED);

    // Blue is 1st in all three die outcomes; White is 2nd; the rest lose.
    assert_close(result[&Camel::Blue], 3.0);
    assert_close(result[&Camel::White], 1.0);
    assert_close(result[&Camel::Yellow], -1.0);
    assert_close(result[&Camel::Orange], -1.0);
    assert_close(result[&Camel::Green], -1.0);
}

#[test]
fn printed_value_is_the_tile_payout_for_first_place() {
    for value in [2, 3, 5] {
        let bets = top_tiles(&[(Camel::Blue, value)]);
        let result = expected_leg_bet_coins(&single_die_state(), &bets, &FOUR_MOVED);
        assert_close(result[&Camel::Blue], value as f64);
    }
}

/// Blue's die can land on the leading stack, making Blue 1st and White 2nd in
/// the *same* stack in one of the three equally likely outcomes.
fn stack_finish_state() -> State {
    state(&[
        (2, vec![Camel::Blue]),
        (3, vec![Camel::Green]),
        (4, vec![Camel::Orange]),
        (5, vec![Camel::Yellow, Camel::White]),
    ])
}

const OTHER_FOUR_MOVED: [Camel; 4] = [Camel::Green, Camel::Orange, Camel::White, Camel::Yellow];

#[test]
fn second_place_in_same_stack_counts_as_second() {
    let bets = top_tiles(&[
        (Camel::Blue, 5),
        (Camel::White, 3),
        (Camel::Yellow, 2),
        (Camel::Orange, 5),
        (Camel::Green, 2),
    ]);

    let result = expected_leg_bet_coins(&stack_finish_state(), &bets, &OTHER_FOUR_MOVED);

    // Blue: P(1st) = 1/3. White: P(1st) = 2/3, P(2nd) = 1/3.
    // Yellow: P(2nd) = 2/3. Orange and Green never place in the top two.
    assert_close(result[&Camel::Blue], (5.0 + 1.0) / 3.0 - 1.0);
    assert_close(result[&Camel::White], 4.0 * 2.0 / 3.0 + 2.0 / 3.0 - 1.0);
    assert_close(result[&Camel::Yellow], 2.0 * 2.0 / 3.0 - 1.0);
    assert_close(result[&Camel::Orange], -1.0);
    assert_close(result[&Camel::Green], -1.0);
}

#[test]
fn camels_without_a_tile_are_empty() {
    let mut bets = top_tiles(&[
        (Camel::Blue, 5),
        (Camel::White, 3),
        (Camel::Yellow, 2),
        (Camel::Green, 2),
    ]);
    bets.insert(Camel::Blue, vec![]);

    let result = expected_leg_bet_coins(&stack_finish_state(), &bets, &OTHER_FOUR_MOVED);

    assert_eq!(result[&Camel::Blue], None);
    assert_eq!(result[&Camel::Orange], None); // missing entry
    assert!(result[&Camel::White].is_some());
    assert!(result[&Camel::Yellow].is_some());
    assert!(result[&Camel::Green].is_some());
}

#[test]
fn exhausted_stacks_still_use_the_top_remaining_tile() {
    let mut bets = top_tiles(&[(Camel::Blue, 5)]);
    bets.insert(Camel::Blue, vec![5, 3, 2]);

    let result = expected_leg_bet_coins(&single_die_state(), &bets, &FOUR_MOVED);

    // Only the top tile (5) is taken, not the sum of the stack.
    assert_close(result[&Camel::Blue], 5.0);
}

#[test]
fn all_dice_moved_scores_the_current_ranking() {
    let values: Vec<(Camel, u8)> = ALL.iter().map(|&camel| (camel, 5)).collect();
    let bets = top_tiles(&values);

    let result = expected_leg_bet_coins(&single_die_state(), &bets, &ALL);

    assert_close(result[&Camel::Blue], 5.0);
    assert_close(result[&Camel::White], 1.0);
    assert_close(result[&Camel::Yellow], -1.0);
    assert_close(result[&Camel::Orange], -1.0);
    assert_close(result[&Camel::Green], -1.0);
}

#[test]
fn finished_race_scores_the_current_ranking() {
    // Blue has crossed space 16, so it is 1st and White is 2nd.
    let finished = state(&[
        (1, vec![Camel::Green]),
        (2, vec![Camel::Orange]),
        (3, vec![Camel::Yellow]),
        (15, vec![Camel::White]),
        (17, vec![Camel::Blue]),
    ]);
    let values: Vec<(Camel, u8)> = ALL.iter().map(|&camel| (camel, 5)).collect();
    let bets = top_tiles(&values);

    let result = expected_leg_bet_coins(&finished, &bets, &[]);

    assert_close(result[&Camel::Blue], 5.0);
    assert_close(result[&Camel::White], 1.0);
    assert_close(result[&Camel::Yellow], -1.0);
}

#[test]
fn crossing_the_finish_line_during_the_leg_is_scored_at_the_crossing() {
    // Blue starts on 16, so its die always crosses the line; the other camels
    // are too far back to join its stack or cross first.
    let state = state(&[
        (1, vec![Camel::Green]),
        (2, vec![Camel::Orange]),
        (3, vec![Camel::Yellow]),
        (12, vec![Camel::White]),
        (16, vec![Camel::Blue]),
    ]);
    let bets = top_tiles(&[(Camel::Blue, 5), (Camel::White, 3)]);

    let result = expected_leg_bet_coins(&state, &bets, &[]);

    assert_close(result[&Camel::Blue], 5.0);
    assert_close(result[&Camel::White], 1.0);
}

#[test]
fn full_leg_is_deterministic_and_higher_tiles_are_no_worse() {
    let low_values: Vec<(Camel, u8)> = ALL.iter().map(|&camel| (camel, 2)).collect();
    let high_values: Vec<(Camel, u8)> = ALL.iter().map(|&camel| (camel, 5)).collect();
    let low = top_tiles(&low_values);
    let high = top_tiles(&high_values);

    let first = expected_leg_bet_coins(&single_die_state(), &low, &[]);
    let second = expected_leg_bet_coins(&single_die_state(), &low, &[]);
    let high_result = expected_leg_bet_coins(&single_die_state(), &high, &[]);

    assert_eq!(first, second);
    for &camel in &ALL {
        let low_ev = first[&camel].expect("tile available");
        let high_ev = high_result[&camel].expect("tile available");
        assert!(low_ev.is_finite() && high_ev.is_finite());
        assert!(
            high_ev >= low_ev,
            "a 5-tile cannot be worse than a 2-tile for {:?}",
            camel
        );
    }
}

#[test]
#[should_panic(expected = "invalid leg-bet tile value")]
fn invalid_tile_value_panics() {
    let bets = top_tiles(&[(Camel::Blue, 4)]);
    expected_leg_bet_coins(&single_die_state(), &bets, &FOUR_MOVED);
}

#[test]
#[should_panic(expected = "must be ordered 5, 3, 2")]
fn unordered_stack_panics() {
    let mut bets = top_tiles(&[(Camel::Blue, 5)]);
    bets.insert(Camel::Blue, vec![2, 5]);
    expected_leg_bet_coins(&single_die_state(), &bets, &FOUR_MOVED);
}

#[test]
#[should_panic(expected = "appears multiple times in already_moved")]
fn duplicate_moved_camel_panics() {
    let bets = top_tiles(&[(Camel::Blue, 5)]);
    expected_leg_bet_coins(
        &single_die_state(),
        &bets,
        &[Camel::White, Camel::White, Camel::Yellow, Camel::Orange],
    );
}
