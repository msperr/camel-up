use std::collections::BTreeMap;

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

/// A field can either contain camels or a desert tile
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Field {
    Camels(Vec<Camel>),
    Desert(DesertTile),
}

/// Game state
#[derive(Clone, Debug)]
pub struct State {
    /// Mapping from integer keys to field contents (camels or desert)
    pub data: BTreeMap<u8, Field>,
}

impl State {
    /// Create a new State from the provided map
    pub fn new(data: BTreeMap<u8, Field>) -> Self {
        State { data }
    }

    /// Move `camel` forward by `steps` (dice result).
    /// Returns (new_state, optional_desert_field_key)
    /// If the moved camel hits a desert tile, the key of that desert tile is returned.
    ///
    /// Panics if:
    /// - `steps < 1`
    /// - `camel` is not found in `data`
    /// - `camel` appears more than once in `data`
    /// - precondition violated: if a desert is adjacent to another desert in the direction moved
    pub fn move_camel(&self, camel: Camel, steps: u8) -> (Self, Option<u8>) {
        if steps == 0 {
            panic!("steps must be >= 1, got 0");
        }

        // Find the unique occurrence of camel: (field, position)
        let mut found: Option<(u8, usize)> = None;
        for (k, field_val) in &self.data {
            if let Field::Camels(v) = field_val {
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

        let (field, position) = match found {
            Some(fp) => fp,
            None => panic!("camel {:?} not found in state.data", camel),
        };

        // Compute destination using checked_add on u8 to preserve u8 semantics.
        let new_field = field.checked_add(steps).unwrap_or_else(|| {
            panic!(
                "moving camel {:?} from field {} by {} steps would overflow u8",
                camel, field, steps
            )
        });

        // Clone map and perform the mutation on the clone
        let mut map = self.data.clone();

        // Remove the source camels vector so we can mutate the map without overlapping borrows
        let mut src_vec = match map.remove(&field) {
            Some(Field::Camels(v)) => v,
            Some(Field::Desert(_)) => {
                panic!("internal error: expected camels at {}, found desert", field)
            }
            None => panic!("internal error: expected key {} present", field),
        };

        // split_off returns the tail starting at `position`
        let tail = src_vec.split_off(position);

        // if there is a non-empty prefix, put it back under the original field
        if !src_vec.is_empty() {
            map.insert(field, Field::Camels(src_vec));
        }

        // Now inspect destination
        match map.get(&new_field) {
            None => {
                // empty field — insert the tail
                map.insert(new_field, Field::Camels(tail));
                (State { data: map }, None)
            }
            Some(Field::Camels(existing)) => {
                // append to existing
                let mut new_vec = existing.clone();
                new_vec.extend(tail);
                map.insert(new_field, Field::Camels(new_vec));
                (State { data: map }, None)
            }
            Some(Field::Desert(_)) => {
                // delegate desert-specific behaviour to helper
                let desert_hit = self.apply_desert_effect(&mut map, new_field, tail);
                (State { data: map }, desert_hit)
            }
        }
    }

    // Apply desert tile effect when a camel tail landed on `desert_field`.
    // Mutates `map` (the cloned/working map) to apply the oasis/mirage behaviour
    // and returns Some(desert_field). Uses `self.data` for the precondition checks.
    fn apply_desert_effect(
        &self,
        map: &mut BTreeMap<u8, Field>,
        desert_field: u8,
        tail: Vec<Camel>,
    ) -> Option<u8> {
        // Look up the tile type from the original State (self.data).
        match self.data.get(&desert_field) {
            Some(Field::Desert(DesertTile::Oasis)) => {
                let forward = desert_field.checked_add(1).unwrap_or_else(|| {
                    panic!(
                        "moving camel from desert {} by oasis forward would overflow u8",
                        desert_field
                    )
                });

                if let Some(Field::Desert(_)) = self.data.get(&forward) {
                    panic!(
                        "precondition violated: desert at {} adjacent to desert at {}",
                        desert_field, forward
                    );
                }

                // append to forward (create or append)
                match map.get(&forward) {
                    Some(Field::Camels(existing_fwd)) => {
                        let mut new_vec = existing_fwd.clone();
                        new_vec.extend(tail);
                        map.insert(forward, Field::Camels(new_vec));
                    }
                    Some(Field::Desert(_)) => unreachable!(),
                    None => {
                        map.insert(forward, Field::Camels(tail));
                    }
                }

                Some(desert_field)
            }
            Some(Field::Desert(DesertTile::Mirage)) => {
                let back = desert_field.checked_sub(1).unwrap_or_else(|| {
                    panic!(
                        "moving camel from desert {} by mirage back would underflow u8",
                        desert_field
                    )
                });

                if let Some(Field::Desert(_)) = self.data.get(&back) {
                    panic!(
                        "precondition violated: desert at {} adjacent to desert at {}",
                        desert_field, back
                    );
                }

                // prepend to back: if camels exist, prepend; otherwise insert
                match map.remove(&back) {
                    Some(Field::Camels(existing_back)) => {
                        let mut new_vec = tail;
                        new_vec.extend(existing_back);
                        map.insert(back, Field::Camels(new_vec));
                    }
                    Some(Field::Desert(_)) => unreachable!(),
                    None => {
                        map.insert(back, Field::Camels(tail));
                    }
                }

                Some(desert_field)
            }
            other => panic!(
                "internal error: expected Desert at {}, found {:?}",
                desert_field, other
            ),
        }
    }

    pub fn move_multiple_camels<I>(&self, moves: I) -> (Self, BTreeMap<u8, usize>)
    where
        I: IntoIterator<Item = (Camel, u8)>,
    {
        let mut state = self.clone();
        let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
        for (camel, steps) in moves.into_iter() {
            let (new_state, maybe_desert) = state.move_camel(camel, steps);
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
    /// - map from field key (u8) -> total desert hit counts across all simulations
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
                    Field::Camels(v) => v.iter().filter(|&&c| c == camel).count(),
                    Field::Desert(_) => 0usize,
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
                // zipped iterator directly. `move_multiple_camels` accepts any IntoIterator.
                let (result, counts_map) =
                    self.move_multiple_camels(perm.iter().cloned().zip(comb.into_iter()));
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
                Field::Camels(v) => v.to_vec(),
                Field::Desert(_) => Vec::new(),
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
        let max_key = *self.data.keys().last().unwrap();
        let bound = std::cmp::max(max_key, 16u8);

        for pos in min_key..=bound {
            for &tile in &[DesertTile::Oasis, DesertTile::Mirage] {
                // feasibility: cannot place on camels or on an existing desert
                match self.data.get(&pos) {
                    Some(Field::Camels(_)) => continue,
                    Some(Field::Desert(_)) => continue,
                    _ => {}
                }

                // adjacent left desert?
                if let Some(left) = pos.checked_sub(1) {
                    if let Some(Field::Desert(_)) = self.data.get(&left) {
                        continue;
                    }
                }
                // adjacent right desert?
                if let Some(right) = pos.checked_add(1) {
                    if let Some(Field::Desert(_)) = self.data.get(&right) {
                        continue;
                    }
                }

                // create clone with desert at pos
                let mut clone_map = self.data.clone();
                clone_map.insert(pos, Field::Desert(tile));
                let clone_state = State::new(clone_map);

                let (_camel_counts, desert_counts) = clone_state.simulate_outcomes();
                let hits = desert_counts.get(&pos).cloned().unwrap_or(0usize);
                result.entry(pos).or_default().insert(tile, hits);
            }
        }

        result
    }
}
