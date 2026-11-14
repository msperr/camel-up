use camel_up::{Camel, DesertTile, Space, State};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

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
    let (counts, _) = base_state().simulate_outcomes(&moved);

    // 3! permutations * 3^3 dice combinations.
    const PARTIAL_TOTAL: usize = 162;
    for &camel in &ALL {
        assert_eq!(total_for(&counts, camel), PARTIAL_TOTAL);
    }
}

#[test]
fn four_camels_moved_simulates_remaining_one() {
    let moved = [Camel::White, Camel::Yellow, Camel::Orange, Camel::Green];
    let (counts, _) = base_state().simulate_outcomes(&moved);

    // 1! permutation * 3^1 dice combinations.
    const PARTIAL_TOTAL: usize = 3;
    for &camel in &ALL {
        assert_eq!(total_for(&counts, camel), PARTIAL_TOTAL);
    }
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
fn all_camels_moved_panics() {
    let state = base_state();
    let all = ALL;

    // A leg ends once all five dice have been revealed, so there is no
    // remaining move to simulate.
    let result = catch_unwind(AssertUnwindSafe(|| state.simulate_outcomes(&all)));
    assert!(result.is_err());
}

#[test]
fn duplicate_already_moved_panics() {
    let state = base_state();
    let moved = [Camel::White, Camel::Orange, Camel::White];

    let result = catch_unwind(AssertUnwindSafe(|| state.simulate_outcomes(&moved)));
    assert!(result.is_err());
}

#[test]
fn missing_camel_in_state_panics() {
    let mut map = base_map();
    map.remove(&6); // remove Blue
    let state = State::new(map);

    let result = catch_unwind(AssertUnwindSafe(|| state.simulate_outcomes(&[])));
    assert!(result.is_err());
}

#[test]
fn duplicate_camel_in_state_panics() {
    let mut map = base_map();
    map.insert(7, Space::Camels(vec![Camel::Blue])); // Blue now appears twice
    let state = State::new(map);

    let result = catch_unwind(AssertUnwindSafe(|| state.simulate_outcomes(&[])));
    assert!(result.is_err());
}
