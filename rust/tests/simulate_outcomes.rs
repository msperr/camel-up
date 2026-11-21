use camel_up::{Camel, DesertTile, Space, State};
use std::collections::BTreeMap;

const ALL: [Camel; 5] = [
    Camel::White,
    Camel::Yellow,
    Camel::Orange,
    Camel::Green,
    Camel::Blue,
];

fn base_map() -> BTreeMap<u8, Space> {
    let mut m = BTreeMap::new();
    m.insert(1, Space::Camels(vec![Camel::White]));
    m.insert(2, Space::Camels(vec![Camel::Yellow, Camel::Orange]));
    m.insert(4, Space::Camels(vec![Camel::Green]));
    m.insert(6, Space::Camels(vec![Camel::Blue]));
    m.insert(8, Space::Desert(DesertTile::Oasis));
    m
}

fn base_state() -> State {
    State::new(base_map())
}

fn total_for(counts: &BTreeMap<Camel, Vec<usize>>, camel: Camel) -> usize {
    counts[&camel].iter().sum()
}

/// Number of simulated scenarios for `remaining` camels: `remaining! * 3^remaining`.
fn scenario_count(remaining: usize) -> usize {
    let factorial: usize = (1..=remaining).product();
    factorial * 3usize.pow(remaining as u32)
}

#[test]
fn no_camels_moved_is_full_leg() {
    let (counts, _) = base_state().simulate_outcomes(&[]);

    // 5! permutations * 3^5 dice combinations.
    const FULL_LEG_TOTAL: usize = 29_160;
    for &camel in &ALL {
        assert_eq!(counts[&camel].len(), ALL.len());
        assert_eq!(total_for(&counts, camel), FULL_LEG_TOTAL);
    }
}

#[test]
fn two_camels_moved_simulates_remaining_three() {
    let moved = [Camel::White, Camel::Yellow];
    // Reflect the already-revealed moves in the state, as the API requires.
    let (state, _) = base_state().move_multiple_units([(Camel::White, 1), (Camel::Yellow, 2)]);
    let (counts, _) = state.simulate_outcomes(&moved);

    // 3! permutations * 3^3 dice combinations.
    const PARTIAL_TOTAL: usize = 162;
    for &camel in &ALL {
        assert_eq!(total_for(&counts, camel), PARTIAL_TOTAL);
    }
}

#[test]
fn four_camels_moved_simulates_remaining_one() {
    let moved = [Camel::White, Camel::Yellow, Camel::Orange, Camel::Green];
    // Reflect the already-revealed moves in the state, as the API requires.
    let (state, _) = base_state().move_multiple_units([
        (Camel::White, 1),
        (Camel::Yellow, 2),
        (Camel::Orange, 3),
        (Camel::Green, 1),
    ]);
    let (counts, _) = state.simulate_outcomes(&moved);

    // 1! permutation * 3^1 dice combinations.
    const PARTIAL_TOTAL: usize = 3;
    for &camel in &ALL {
        assert_eq!(total_for(&counts, camel), PARTIAL_TOTAL);
    }
}

#[test]
fn enumeration_handles_a_finish_during_the_scenario() {
    let moved = [Camel::Yellow, Camel::Orange, Camel::Green, Camel::Blue];
    let mut data = BTreeMap::new();
    data.insert(1, Space::Camels(vec![Camel::Yellow]));
    data.insert(2, Space::Camels(vec![Camel::Orange]));
    data.insert(3, Space::Camels(vec![Camel::Green]));
    data.insert(4, Space::Camels(vec![Camel::Blue]));
    data.insert(15, Space::Camels(vec![Camel::White]));
    data.insert(16, Space::Desert(DesertTile::Oasis));
    let state = State::new(data);

    // Only White remains. Die 1 hits the oasis on 16 and crosses via its extra
    // step; dice 2 and 3 cross directly without triggering the desert.
    let (counts, desert_hits) = state.simulate_outcomes(&moved);

    let scenarios = scenario_count(1);
    assert_eq!(scenarios, 3);
    assert_eq!(total_for(&counts, Camel::White), scenarios);
    assert_eq!(counts[&Camel::White][0], scenarios, "crossing camel is 1st");

    // The desert is paid only in the die-1 scenario.
    assert_eq!(desert_hits.get(&16), Some(&1));
}

#[test]
fn partial_composition_matches_full_leg() {
    let state = base_state();
    let (full_counts, full_desert) = state.simulate_outcomes(&[]);

    let mut aggregated_counts: BTreeMap<Camel, Vec<usize>> = ALL
        .iter()
        .map(|&camel| (camel, vec![0; ALL.len()]))
        .collect();
    let mut aggregated_desert: BTreeMap<u8, usize> = BTreeMap::new();

    // Each first move is followed by this many continuations (4! * 3^4).
    let continuations_per_first_move = scenario_count(ALL.len() - 1);

    // Group every full-leg scenario by its first move, then let
    // `simulate_outcomes` enumerate the remaining four camels.
    for &first in &ALL {
        for die in 1..=3u8 {
            let (after_first, first_desert_hit) = state.move_unit(first, die);
            if let Some(key) = first_desert_hit {
                *aggregated_desert.entry(key).or_insert(0) += continuations_per_first_move;
            }

            let (partial_counts, partial_desert) = after_first.simulate_outcomes(&[first]);

            for &camel in &ALL {
                for (pos, &value) in partial_counts[&camel].iter().enumerate() {
                    aggregated_counts.get_mut(&camel).unwrap()[pos] += value;
                }
            }
            for (key, value) in partial_desert {
                *aggregated_desert.entry(key).or_insert(0) += value;
            }
        }
    }

    assert_eq!(aggregated_counts, full_counts);
    assert_eq!(aggregated_desert, full_desert);
}

#[test]
#[should_panic(expected = "all camels have already moved")]
fn all_camels_moved_panics() {
    let state = base_state();
    let all = ALL;

    // A leg ends once all five dice have been revealed, so there is no
    // remaining move to simulate.
    state.simulate_outcomes(&all);
}

#[test]
#[should_panic(expected = "appears multiple times in already_moved")]
fn duplicate_already_moved_panics() {
    let state = base_state();
    let moved = [Camel::White, Camel::Orange, Camel::White];

    state.simulate_outcomes(&moved);
}

#[test]
#[should_panic(expected = "camel Blue appears 0 times")]
fn missing_camel_in_state_panics() {
    let mut map = base_map();
    map.remove(&6); // remove Blue
    let state = State::new(map);

    state.simulate_outcomes(&[]);
}

#[test]
#[should_panic(expected = "camel Blue appears 2 times")]
fn duplicate_camel_in_state_panics() {
    let mut map = base_map();
    map.insert(7, Space::Camels(vec![Camel::Blue])); // Blue now appears twice
    let state = State::new(map);

    state.simulate_outcomes(&[]);
}

#[test]
fn ranking_is_first_place_to_last_place() {
    // Advanced stack at 3: [Green, Blue] (Blue on top); rear stack at 1.
    let mut map = BTreeMap::new();
    map.insert(1, Space::Camels(vec![Camel::White, Camel::Yellow]));
    map.insert(3, Space::Camels(vec![Camel::Green, Camel::Blue]));
    let state = State::new(map);

    assert_eq!(
        state.order(),
        vec![Camel::White, Camel::Yellow, Camel::Green, Camel::Blue]
    );
    assert_eq!(
        state.ranking(),
        vec![Camel::Blue, Camel::Green, Camel::Yellow, Camel::White]
    );
}

#[test]
fn outcome_vectors_are_indexed_by_finishing_position() {
    // Only Blue remains: wherever its die takes it (3, 4 or 5) it is ahead of
    // the stack on space 1, so it must be 1st in all three scenarios.
    let mut map = BTreeMap::new();
    map.insert(
        1,
        Space::Camels(vec![
            Camel::White,
            Camel::Yellow,
            Camel::Orange,
            Camel::Green,
        ]),
    );
    map.insert(2, Space::Camels(vec![Camel::Blue]));
    let state = State::new(map);

    let moved = [Camel::White, Camel::Yellow, Camel::Orange, Camel::Green];
    let (counts, _) = state.simulate_outcomes(&moved);

    assert_eq!(counts[&Camel::Blue][0], 3);
    assert_eq!(counts[&Camel::Blue][1..].iter().sum::<usize>(), 0);
    // Inside the stack at space 1 the ranking runs from the top down.
    assert_eq!(counts[&Camel::Green][1], 3);
    assert_eq!(counts[&Camel::Orange][2], 3);
    assert_eq!(counts[&Camel::Yellow][3], 3);
    assert_eq!(counts[&Camel::White][4], 3);
}

#[test]
#[should_panic(expected = "race has already finished")]
fn simulating_a_finished_race_panics() {
    let mut map = BTreeMap::new();
    map.insert(
        16,
        Space::Camels(vec![
            Camel::White,
            Camel::Yellow,
            Camel::Orange,
            Camel::Green,
            Camel::Blue,
        ]),
    );
    let state = State::new(map);
    let (finished, _) = state.move_unit(Camel::White, 1);

    finished.simulate_outcomes(&[]);
}
