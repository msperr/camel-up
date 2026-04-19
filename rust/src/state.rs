use std::collections::BTreeMap;

// Maximum field index (inclusive)
const MAX_FIELD: u8 = 16;

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

/// Game state
#[derive(Clone, Debug)]
pub struct State {
    /// Mapping from integer keys to space contents (camels or desert)
    pub data: BTreeMap<u8, Space>,
}

impl State {
    /// Create a new State from the provided map
    pub fn new(data: BTreeMap<u8, Space>) -> Self {
        State { data }
    }

    /// Move `camel` forward by `steps` (dice result).
    /// Returns (new_state, optional_desert_space_key)
    /// If the moved camel unit hits a desert tile, the key of that desert tile is returned.
    ///
    /// Panics if:
    /// - `steps < 1`
    /// - `camel` is not found in `data`
    /// - `camel` appears more than once in `data`
    /// - precondition violated: if a desert is adjacent to another desert in the direction moved
    pub fn move_unit(&self, camel: Camel, steps: u8) -> (Self, Option<u8>) {
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

                if let Some(Space::Desert(_)) = self.data.get(&forward_space) {
                    panic!(
                        "precondition violated: desert at {} adjacent to desert at {}",
                        desert_space, forward_space
                    );
                }

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

                if let Some(Space::Desert(_)) = self.data.get(&back_space) {
                    panic!(
                        "precondition violated: desert at {} adjacent to desert at {}",
                        desert_space, back_space
                    );
                }

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

    pub fn move_multiple_units<I>(&self, moves: I) -> (Self, BTreeMap<u8, usize>)
    where
        I: IntoIterator<Item = (Camel, u8)>,
    {
        let mut state = self.clone();
        let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
        for (camel, steps) in moves.into_iter() {
            let (new_state, maybe_desert) = state.move_unit(camel, steps);
            if let Some(k) = maybe_desert {
                *counts.entry(k).or_insert(0) += 1usize;
            }
            state = new_state;
        }
        (state, counts)
    }

    /// Simulate exhaustive outcomes by running all permutations of camels and all combinations of dice
    /// choices = [1,2,3] repeated `num_camels` times.
    /// Returns a tuple of:
    /// - map from Camel -> counts per position (Vec indexed by final position)
    /// - map from space key (u8) -> total desert hit counts across all simulations
    pub fn simulate_outcomes(
        &self,
    ) -> (
        std::collections::BTreeMap<Camel, Vec<usize>>,
        std::collections::BTreeMap<u8, usize>,
    ) {
        use crate::combinatorics::{Permutations, Product};

        let camel_list = vec![
            Camel::White,
            Camel::Yellow,
            Camel::Orange,
            Camel::Green,
            Camel::Blue,
        ];
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
                    "tally_outcomes requires each Camel to appear exactly once in state.data; camel {:?} appears {} times",
                    camel, occurrences
                );
            }
        }
        let num_camels = camel_list.len();
        let choices: Vec<u8> = vec![1, 2, 3];

        let mut counts = std::collections::BTreeMap::new();
        for &c in &camel_list {
            counts.insert(c, vec![0usize; num_camels]);
        }

        let mut desert_counts: std::collections::BTreeMap<u8, usize> =
            std::collections::BTreeMap::new();

        for perm in Permutations::new(camel_list.clone()) {
            for comb in Product::new(choices.clone(), num_camels) {
                // Avoid allocating a Vec for each permutation+combination by passing the
                // zipped iterator directly. `move_multiple_units` accepts any IntoIterator.
                let (result, counts_map) =
                    self.move_multiple_units(perm.iter().cloned().zip(comb.into_iter()));
                let order = result.order();
                for (pos, &camel) in order.iter().enumerate() {
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

    /// Return all camels flattened in order by the map's key.
    /// This mirrors: [c for _, y in sorted(self.data.items(), key=lambda s: s[0]) for c in y][::1]
    pub fn order(&self) -> Vec<Camel> {
        self.data
            .iter()
            .flat_map(|(_, f)| match f {
                Space::Camels(v) => v.to_vec(),
                Space::Desert(_) => Vec::new(),
            })
            .collect()
    }

    /// Evaluate desert placements: for each position and desert tile type, compute how many
    /// desert hits would occur if that desert tile were placed at that position.
    /// Returns a map: position -> (DesertTile -> hit_count)
    pub fn evaluate_desert_placements(
        &self,
    ) -> std::collections::BTreeMap<u8, std::collections::BTreeMap<DesertTile, usize>> {
        let mut result: std::collections::BTreeMap<
            u8,
            std::collections::BTreeMap<DesertTile, usize>,
        > = std::collections::BTreeMap::new();

        if self.data.is_empty() {
            return result;
        }

        let min_key = *self.data.keys().next().unwrap();
        if min_key > MAX_FIELD {
            return result;
        }
        let bound = MAX_FIELD;

        for pos in min_key..=bound {
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

                let (_camel_counts, desert_counts) = clone_state.simulate_outcomes();
                let hits = desert_counts.get(&pos).cloned().unwrap_or(0usize);
                result.entry(pos).or_default().insert(tile, hits);
            }
        }

        result
    }
}
