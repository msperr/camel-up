use std::collections::BTreeMap;

/// Highest numbered space on the race track (inclusive). Crossing it ends the race.
pub const MAX_FIELD: u8 = 16;

/// Camel colors
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Camel {
    White,
    Yellow,
    Orange,
    Green,
    Blue,
}

/// Desert tile types
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DesertTile {
    Oasis,
    Mirage,
}

/// A space can either contain camels (a camel unit/stack) or a desert tile
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Space {
    Camels(Vec<Camel>),
    Desert(DesertTile),
}

/// Panic when `neighbour` holds a desert tile adjacent to `desert_space`.
fn assert_no_adjacent_desert(data: &BTreeMap<u8, Space>, desert_space: u8, neighbour: u8) {
    if let Some(Space::Desert(_)) = data.get(&neighbour) {
        panic!(
            "precondition violated: desert at {} adjacent to desert at {}",
            desert_space, neighbour
        );
    }
}

/// Game state
#[derive(Clone, Debug)]
pub struct State {
    /// Mapping from integer keys to space contents (camels or desert).
    ///
    /// Desert tiles only ever occupy track spaces `1..=MAX_FIELD`. Keys above
    /// [`MAX_FIELD`] hold camel units that have crossed the finish line; they
    /// are an off-board representation, not additional track spaces.
    pub data: BTreeMap<u8, Space>,
}

impl State {
    /// Create a new State from the provided map.
    pub fn new(data: BTreeMap<u8, Space>) -> Self {
        State { data }
    }

    /// Whether the race has ended (a camel unit crossed the finish line).
    ///
    /// A crossing unit is stored at a key greater than [`MAX_FIELD`], so the
    /// finish condition is fully reconstructible from [`State::data`].
    pub fn is_finished(&self) -> bool {
        self.data
            .iter()
            .any(|(&key, space)| key > MAX_FIELD && matches!(space, Space::Camels(_)))
    }

    /// Move `camel` forward by `steps` (dice result).
    /// Returns (new_state, optional_desert_space_key)
    /// If the moved camel unit hits a desert tile, the key of that desert tile is returned.
    ///
    /// A move that would take the unit past space 16 ends the race. The unit is
    /// placed off the board beyond the finish line (at the dice destination,
    /// or at space 17 when an oasis on 16 pushes it over), where it still ranks
    /// ahead of every camel on the track. No further moves can then be made.
    /// A desert tile on 16 that was not triggered stays on the board; an oasis
    /// that was hit is reported as a desert hit.
    ///
    /// Panics if:
    /// - the race has already finished
    /// - `steps < 1`
    /// - `camel` is not found in `data`
    /// - `camel` appears more than once in `data`
    /// - precondition violated: if a desert is adjacent to another desert in the direction moved
    pub fn move_unit(&self, camel: Camel, steps: u8) -> (Self, Option<u8>) {
        if self.is_finished() {
            panic!("cannot move a unit after the race has finished");
        }
        if steps == 0 {
            panic!("steps must be >= 1, got 0");
        }

        // Find the unique occurrence of camel: (src_space, idx)
        let mut found: Option<(u8, usize)> = None;
        for (k, space_val) in &self.data {
            if let Space::Camels(v) = space_val {
                for (i, &c) in v.iter().enumerate() {
                    if c == camel {
                        if found.is_some() {
                            panic!("camel {:?} appears multiple times in data", camel);
                        }
                        found = Some((*k, i));
                    }
                }
            }
        }

        let (src_space, idx) = match found {
            Some(fp) => fp,
            None => panic!("camel {:?} not found in state.data", camel),
        };

        // Compute destination using checked_add on u8 to preserve u8 semantics.
        let dest_space = src_space.checked_add(steps).unwrap_or_else(|| {
            panic!(
                "moving camel {:?} from space {} by {} steps would overflow u8",
                camel, src_space, steps
            )
        });

        // Clone board and perform the mutation on the clone
        let mut board = self.data.clone();

        // Remove the source camels vector so we can mutate the map without overlapping borrows
        let mut src_stack = match board.remove(&src_space) {
            Some(Space::Camels(v)) => v,
            Some(Space::Desert(_)) => {
                panic!(
                    "internal error: expected camels at {}, found desert",
                    src_space
                )
            }
            None => panic!("internal error: expected key {} present", src_space),
        };

        // split_off returns the moving unit starting at `idx`
        let moving_unit = src_stack.split_off(idx);

        // if there is a non-empty prefix, put it back under the original space
        if !src_stack.is_empty() {
            board.insert(src_space, Space::Camels(src_stack));
        }

        // The unit crosses the line if the dice move lands past space 16, or if
        // an oasis on space 16 would push it one space further.
        let oasis_at_dest = matches!(
            board.get(&dest_space),
            Some(Space::Desert(DesertTile::Oasis))
        );
        let crosses_finish = dest_space > MAX_FIELD || (dest_space == MAX_FIELD && oasis_at_dest);

        if crosses_finish {
            // The extra oasis step also crosses the line when the tile is on
            // space 16. Keys above MAX_FIELD are off-board, so no desert tile
            // can be present there in a legal state.
            let finish_space = if oasis_at_dest {
                dest_space
                    .checked_add(1)
                    .expect("oasis at space 16 cannot overflow u8")
            } else {
                dest_space
            };
            let desert_hit = if oasis_at_dest {
                Some(dest_space)
            } else {
                None
            };

            // Place the whole unit off the track; its key makes it rank first.
            match board.get(&finish_space) {
                Some(Space::Camels(existing)) => {
                    let mut new_vec = existing.clone();
                    new_vec.extend(moving_unit);
                    board.insert(finish_space, Space::Camels(new_vec));
                }
                _ => {
                    board.insert(finish_space, Space::Camels(moving_unit));
                }
            }
            return (State { data: board }, desert_hit);
        }

        // Now inspect destination
        match board.get(&dest_space) {
            None => {
                // empty space — insert the moving unit
                board.insert(dest_space, Space::Camels(moving_unit));
                (State { data: board }, None)
            }
            Some(Space::Camels(existing)) => {
                // append to existing stack
                let mut new_vec = existing.clone();
                new_vec.extend(moving_unit);
                board.insert(dest_space, Space::Camels(new_vec));
                (State { data: board }, None)
            }
            Some(Space::Desert(_)) => {
                // delegate desert-specific behaviour to helper
                let desert_hit = self.apply_desert_effect(&mut board, dest_space, moving_unit);
                (State { data: board }, desert_hit)
            }
        }
    }

    // Apply desert tile effect when a camel unit landed on `desert_space`.
    // Mutates `board` (the cloned/working map) to apply the oasis/mirage behaviour
    // and returns Some(desert_space). Uses `self.data` for the precondition checks.
    fn apply_desert_effect(
        &self,
        board: &mut BTreeMap<u8, Space>,
        desert_space: u8,
        moving_unit: Vec<Camel>,
    ) -> Option<u8> {
        // Look up the tile type from the original State (self.data).
        match self.data.get(&desert_space) {
            Some(Space::Desert(DesertTile::Oasis)) => {
                let forward_space = desert_space.checked_add(1).unwrap_or_else(|| {
                    panic!(
                        "moving camel from desert {} by oasis forward would overflow u8",
                        desert_space
                    )
                });

                assert_no_adjacent_desert(&self.data, desert_space, forward_space);

                // append to forward (create or append)
                match board.get(&forward_space) {
                    Some(Space::Camels(existing_fwd)) => {
                        let mut new_vec = existing_fwd.clone();
                        new_vec.extend(moving_unit);
                        board.insert(forward_space, Space::Camels(new_vec));
                    }
                    Some(Space::Desert(_)) => unreachable!(),
                    None => {
                        board.insert(forward_space, Space::Camels(moving_unit));
                    }
                }

                Some(desert_space)
            }
            Some(Space::Desert(DesertTile::Mirage)) => {
                let back_space = desert_space.checked_sub(1).unwrap_or_else(|| {
                    panic!(
                        "moving camel from desert {} by mirage back would underflow u8",
                        desert_space
                    )
                });

                assert_no_adjacent_desert(&self.data, desert_space, back_space);

                // prepend to back: if camels exist, prepend; otherwise insert
                match board.remove(&back_space) {
                    Some(Space::Camels(existing_back)) => {
                        let mut new_vec = moving_unit;
                        new_vec.extend(existing_back);
                        board.insert(back_space, Space::Camels(new_vec));
                    }
                    Some(Space::Desert(_)) => unreachable!(),
                    None => {
                        board.insert(back_space, Space::Camels(moving_unit));
                    }
                }

                Some(desert_space)
            }
            other => panic!(
                "internal error: expected Desert at {}, found {:?}",
                desert_space, other
            ),
        }
    }

    /// Apply moves in order, accumulating desert hit counts per space.
    ///
    /// The race stops immediately once a camel unit crosses the finish line,
    /// so any remaining moves in `moves` are ignored.
    pub fn move_multiple_units<I>(&self, moves: I) -> (Self, BTreeMap<u8, usize>)
    where
        I: IntoIterator<Item = (Camel, u8)>,
    {
        let mut state = self.clone();
        let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
        for (camel, steps) in moves.into_iter() {
            if state.is_finished() {
                break;
            }
            let (new_state, maybe_desert) = state.move_unit(camel, steps);
            if let Some(k) = maybe_desert {
                *counts.entry(k).or_insert(0) += 1usize;
            }
            state = new_state;
        }
        (state, counts)
    }

    /// Simulate exhaustive outcomes by running all permutations of the camels
    /// that have not moved yet and all combinations of their dice (1, 2 or 3).
    ///
    /// `already_moved` contains the camels whose dice have already been revealed
    /// during the current leg. `self` must already reflect those moves; the
    /// listed camels are excluded from the simulation and only the remaining
    /// camels are permuted and rolled. Passing an empty slice reproduces the
    /// full-leg simulation.
    ///
    /// Rules note: a leg ends once all five racing dice have been revealed. At
    /// that point the dice are returned to the pyramid and a new leg begins, so
    /// there are no moves left to simulate. Passing all five camels is therefore
    /// invalid.
    ///
    /// Returns a tuple of:
    /// - map from Camel -> counts per finishing position. Index 0 is **1st
    ///   place** and index 4 is last, matching [`State::ranking`].
    /// - map from space key (u8) -> total desert hit counts across all simulations
    ///
    /// When `already_moved` is non-empty, the desert counts cover only the hits
    /// caused by the remaining simulated moves. Hits from the already-revealed
    /// moves are not included and must be accounted for by the caller.
    ///
    /// Panics if:
    /// - the race has already finished
    /// - `already_moved` contains a camel more than once
    /// - all five camels have already moved (the leg is over)
    /// - any Camel appears more or less than once in `self.data`
    pub fn simulate_outcomes(
        &self,
        already_moved: &[Camel],
    ) -> (
        std::collections::BTreeMap<Camel, Vec<usize>>,
        std::collections::BTreeMap<u8, usize>,
    ) {
        use crate::combinatorics::{Permutations, Product};

        if self.is_finished() {
            panic!("simulate_outcomes: the race has already finished");
        }

        let camel_list = vec![
            Camel::White,
            Camel::Yellow,
            Camel::Orange,
            Camel::Green,
            Camel::Blue,
        ];

        // Reject duplicate entries: each camel's die can only be revealed once
        // per leg.
        for (i, &camel) in already_moved.iter().enumerate() {
            if already_moved[..i].contains(&camel) {
                panic!(
                    "simulate_outcomes: camel {:?} appears multiple times in already_moved",
                    camel
                );
            }
        }

        // Preconditions: every Camel must appear exactly once in self.data
        for &camel in &camel_list {
            let occurrences: usize = self
                .data
                .values()
                .map(|f| match f {
                    Space::Camels(v) => v.iter().filter(|&&c| c == camel).count(),
                    Space::Desert(_) => 0usize,
                })
                .sum();
            if occurrences != 1 {
                panic!(
                    "simulate_outcomes requires each Camel to appear exactly once in state.data; camel {:?} appears {} times",
                    camel, occurrences
                );
            }
        }

        // A leg is over as soon as the last camel has moved: the dice go back
        // into the pyramid and there is nothing left to simulate.
        let remaining: Vec<Camel> = camel_list
            .iter()
            .copied()
            .filter(|camel| !already_moved.contains(camel))
            .collect();
        if remaining.is_empty() {
            panic!(
                "simulate_outcomes: all camels have already moved; the leg is over and there are no outcomes left to simulate"
            );
        }

        let num_camels = camel_list.len();
        let num_remaining = remaining.len();
        let choices: Vec<u8> = vec![1, 2, 3];

        let mut counts = std::collections::BTreeMap::new();
        for &c in &camel_list {
            counts.insert(c, vec![0usize; num_camels]);
        }

        let mut desert_counts: std::collections::BTreeMap<u8, usize> =
            std::collections::BTreeMap::new();

        for perm in Permutations::new(remaining) {
            for comb in Product::new(choices.clone(), num_remaining) {
                // Avoid allocating a Vec for each permutation+combination by passing the
                // zipped iterator directly. `move_multiple_units` accepts any IntoIterator.
                let (result, counts_map) = self.move_multiple_units(perm.iter().cloned().zip(comb));
                let ranking = result.ranking();
                for (pos, &camel) in ranking.iter().enumerate() {
                    if let Some(v) = counts.get_mut(&camel) {
                        v[pos] += 1;
                    }
                }
                // accumulate desert hit counts per position
                for (k, &v) in &counts_map {
                    *desert_counts.entry(*k).or_insert(0) += v;
                }
            }
        }

        (counts, desert_counts)
    }

    /// Return all camels flattened by board position (rear to front), then by
    /// height within each stack (bottom to top).
    ///
    /// This mirrors: [c for _, y in sorted(self.data.items(), key=lambda s: s[0]) for c in y]
    pub fn order(&self) -> Vec<Camel> {
        self.data
            .values()
            .flat_map(|f| match f {
                Space::Camels(v) => v.to_vec(),
                Space::Desert(_) => Vec::new(),
            })
            .collect()
    }

    /// Return the camels in race order: index 0 is 1st place, the last index is
    /// the overall/leg loser.
    ///
    /// Ranking is by board position, then by height in the stack: the top camel
    /// of the most advanced stack is 1st, the camel directly below it is 2nd,
    /// and so on (RULES.md section 2).
    pub fn ranking(&self) -> Vec<Camel> {
        self.order().into_iter().rev().collect()
    }

    /// Evaluate desert placements: for each evaluated empty space at or ahead of
    /// the rearmost camel, and for each desert tile type, compute how many
    /// desert hits would occur if that tile were placed there. Spaces behind
    /// the rearmost camel are excluded because camels can never move backwards
    /// far enough to reach them.
    ///
    /// Returns a map: position -> (DesertTile -> hit_count). An empty map is
    /// returned once the race has finished.
    pub fn evaluate_desert_placements(
        &self,
    ) -> std::collections::BTreeMap<u8, std::collections::BTreeMap<DesertTile, usize>> {
        let mut result: std::collections::BTreeMap<
            u8,
            std::collections::BTreeMap<DesertTile, usize>,
        > = std::collections::BTreeMap::new();

        if self.is_finished() || self.data.is_empty() {
            return result;
        }

        // Start from the rearmost camel, not the lowest occupied key: an
        // existing desert behind it is inert and must not widen the range.
        let min_key = match self
            .data
            .iter()
            .filter_map(|(&key, space)| matches!(space, Space::Camels(_)).then_some(key))
            .min()
        {
            Some(key) => key,
            None => return result,
        };
        let bound = MAX_FIELD;

        for pos in min_key..=bound {
            // A desert tile may never be placed on space 1 (RULES.md section 6).
            if pos == 1 {
                continue;
            }
            for &tile in &[DesertTile::Oasis, DesertTile::Mirage] {
                // feasibility: cannot place on camels or on an existing desert
                match self.data.get(&pos) {
                    Some(Space::Camels(_)) => continue,
                    Some(Space::Desert(_)) => continue,
                    _ => {}
                }

                // adjacent left desert?
                if let Some(left) = pos.checked_sub(1) {
                    if let Some(Space::Desert(_)) = self.data.get(&left) {
                        continue;
                    }
                }
                // adjacent right desert?
                if let Some(right) = pos.checked_add(1) {
                    if let Some(Space::Desert(_)) = self.data.get(&right) {
                        continue;
                    }
                }

                // create clone with desert at pos
                let mut clone_map = self.data.clone();
                clone_map.insert(pos, Space::Desert(tile));
                let clone_state = State::new(clone_map);

                let (_camel_counts, desert_counts) = clone_state.simulate_outcomes(&[]);
                let hits = desert_counts.get(&pos).cloned().unwrap_or(0usize);
                result.entry(pos).or_default().insert(tile, hits);
            }
        }

        result
    }
}
