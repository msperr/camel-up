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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub data: BTreeMap<i32, Field>,
}

impl State {
    /// Create a new State from the provided map
    pub fn new(data: BTreeMap<i32, Field>) -> Self {
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
    pub fn move_camel(&self, camel: Camel, steps: i32) -> (Self, Option<i32>) {
        if steps < 1 {
            panic!("steps must be >= 1, got {}", steps);
        }

        // Find the unique occurrence of camel: (field, position)
        let mut found: Option<(i32, usize)> = None;
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

        // Use checked_add to detect overflow when adding steps to field.
        // This panics with a clear message if arithmetic would overflow i32.
        let new_field = field.checked_add(steps).unwrap_or_else(|| {
            panic!(
                "moving camel {:?} from field {} by {} steps overflows i32",
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
        map: &mut BTreeMap<i32, Field>,
        desert_field: i32,
        tail: Vec<Camel>,
    ) -> Option<i32> {
        // Look up the tile type from the original State (self.data).
        match self.data.get(&desert_field) {
            Some(Field::Desert(DesertTile::Oasis)) => {
                let forward = desert_field.checked_add(1).unwrap_or_else(|| {
                    panic!(
                        "moving camel from desert {} by oasis forward would overflow",
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
                        "moving camel from desert {} by mirage back would underflow",
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

    pub fn move_multiple_camels<I>(&self, moves: I) -> (Self, BTreeMap<i32, usize>)
    where
        I: IntoIterator<Item = (Camel, i32)>,
    {
        let mut state = self.clone();
        let mut counts: BTreeMap<i32, usize> = BTreeMap::new();
        for (camel, steps) in moves.into_iter() {
            let (new_state, maybe_desert) = state.move_camel(camel, steps);
            if let Some(k) = maybe_desert {
                *counts.entry(k).or_insert(0) += 1usize;
            }
            state = new_state;
        }
        (state, counts)
    }

    /// Tally outcomes by running all permutations of camels and all combinations of dice
    /// choices = [1,2,3] repeated `num_camels` times. Returns a map from Camel -> counts per position.
    pub fn tally_outcomes(&self) -> std::collections::BTreeMap<Camel, Vec<usize>> {
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
        let choices = vec![1_i32, 2, 3];

        let mut counts = std::collections::BTreeMap::new();
        for &c in &camel_list {
            counts.insert(c, vec![0usize; num_camels]);
        }

        for perm in Permutations::new(camel_list.clone()) {
            for comb in Product::new(choices.clone(), num_camels) {
                // Avoid allocating a Vec for each permutation+combination by passing the
                // zipped iterator directly. `move_multiple_camels` accepts any IntoIterator.
                let (result, _counts) =
                    self.move_multiple_camels(perm.iter().cloned().zip(comb.into_iter()));
                let order = result.order();
                for (pos, &camel) in order.iter().enumerate() {
                    if let Some(v) = counts.get_mut(&camel) {
                        v[pos] += 1;
                    }
                }
            }
        }

        counts
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
}
