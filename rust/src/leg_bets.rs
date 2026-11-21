//! Expected coin (Egyptian Pound, EP) payout for taking a leg-bet tile.
//!
//! RULES.md section 5 defines the payout of one leg-betting tile at leg scoring:
//! the printed value (5, 3 or 2) if the camel is 1st, +1 if it is 2nd, and −1
//! otherwise. With `p1` and `p2` from [`State::simulate_outcomes`], the expected
//! value of a tile with printed value `v` is therefore
//! `v * p1 + 1 * p2 - 1 * (1 - p1 - p2) = (v + 1) * p1 + 2 * p2 - 1`.
//!
//! This module does not model players, turn order, or tiles already held: it
//! answers only what one additional tile is worth from the current board. It
//! also assumes no further player actions (such as placing a desert tile)
//! happen before the remaining dice are revealed; that matches the crate's
//! board-only model, where only camel movement is simulated.

use crate::simulation::CAMEL_ORDER;
use crate::{Camel, State};
use std::collections::BTreeMap;

/// Legal tile values in stack order, top first (RULES.md section 3).
const LEG_BET_VALUES: [u8; 3] = [5, 3, 2];

/// Tiles still available in each camel's leg-bet stack, top tile first.
///
/// Stacks are built 5, 3, 2 from the top (RULES.md section 3), so entries must
/// form a contiguous run of those values in descending order. An empty vector
/// or a missing entry means that camel has no tile left; only the first (top)
/// value is used.
pub type AvailableLegBets = BTreeMap<Camel, Vec<u8>>;

/// Expected EP of taking each camel's top available leg-bet tile.
///
/// `state` must already reflect the moves of the camels in `already_moved`;
/// the remaining racing dice are the other camels. Values are exact: the
/// remaining dice order and each die's 1/2/3 result are enumerated exhaustively
/// via [`State::simulate_outcomes`]. A camel with no available tile maps to
/// `None`.
///
/// If the race has already finished, or all five dice have been revealed, the
/// payout is derived deterministically from the current [`State::ranking`].
/// Unfinished states are enumerated exhaustively; the shared movement model
/// stops a scenario at a finish-line crossing and ranks the camels there.
///
/// Panics if:
/// - `already_moved` contains a camel more than once or has more than five entries
/// - an available tile value is not 5, 3 or 2, or a stack is not in 5/3/2 order
/// - the state is not a legal racing state (see [`State::simulate_outcomes`])
pub fn expected_leg_bet_coins(
    state: &State,
    available: &AvailableLegBets,
    already_moved: &[Camel],
) -> BTreeMap<Camel, Option<f64>> {
    for (i, &camel) in already_moved.iter().enumerate() {
        assert!(
            !already_moved[..i].contains(&camel),
            "expected_leg_bet_coins: camel {:?} appears multiple times in already_moved",
            camel
        );
    }
    assert!(
        already_moved.len() <= CAMEL_ORDER.len(),
        "expected_leg_bet_coins: already_moved has more than five entries"
    );

    for (&camel, tiles) in available {
        for &value in tiles {
            assert!(
                LEG_BET_VALUES.contains(&value),
                "expected_leg_bet_coins: invalid leg-bet tile value {} for camel {:?}",
                value,
                camel
            );
        }
        assert!(
            tiles.is_empty()
                || LEG_BET_VALUES
                    .windows(tiles.len())
                    .any(|window| window == tiles.as_slice()),
            "expected_leg_bet_coins: tiles for camel {:?} must be ordered 5, 3, 2",
            camel
        );
    }

    let (counts, total) = if state.is_finished() || already_moved.len() == CAMEL_ORDER.len() {
        // No dice left, or a camel has crossed the finish line: the current
        // ranking is final and there is nothing left to simulate.
        let ranking = state.ranking();
        assert_eq!(
            ranking.len(),
            CAMEL_ORDER.len(),
            "expected_leg_bet_coins: state must contain all five camels"
        );
        for &camel in &CAMEL_ORDER {
            assert_eq!(
                ranking.iter().filter(|&&c| c == camel).count(),
                1,
                "expected_leg_bet_coins: camel {:?} must appear exactly once in state",
                camel
            );
        }

        let mut deterministic = BTreeMap::new();
        for (pos, &camel) in ranking.iter().enumerate() {
            let mut positions = vec![0usize; CAMEL_ORDER.len()];
            positions[pos] = 1;
            deterministic.insert(camel, positions);
        }
        (deterministic, 1usize)
    } else {
        let (counts, _) = state.simulate_outcomes(already_moved);
        let total: usize = counts
            .values()
            .next()
            .map(|positions| positions.iter().sum())
            .unwrap_or(0);
        (counts, total)
    };

    CAMEL_ORDER
        .iter()
        .map(|&camel| {
            let expected = available
                .get(&camel)
                .and_then(|tiles| tiles.first())
                .map(|&value| {
                    let p1 = counts[&camel][0] as f64 / total as f64;
                    let p2 = counts[&camel][1] as f64 / total as f64;
                    (value as f64 + 1.0) * p1 + 2.0 * p2 - 1.0
                });
            (camel, expected)
        })
        .collect()
}
