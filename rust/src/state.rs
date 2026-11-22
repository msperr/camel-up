use crate::simulation::CAMEL_ORDER;
use std::collections::BTreeMap;

/// Highest numbered space on the race track (inclusive). Crossing it ends the race.
pub const MAX_FIELD: u8 = 16;

/// Track spaces plus the off-board spaces a dice move from space 16 can reach.
const BOARD_SPACES: usize = MAX_FIELD as usize + 4;

fn camel_index(camel: Camel) -> usize {
    CAMEL_ORDER
        .iter()
        .position(|&c| c == camel)
        .expect("camel must be one of the five racing camels")
}

fn camel_from_index(index: usize) -> Camel {
    CAMEL_ORDER[index]
}

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

/// Compact, heap-free board used by the exhaustive simulators.
///
/// Every move in [`State::move_unit`] clones the whole `BTreeMap` and the
/// touched stack `Vec`s. A leg simulation repeats that millions of times, so
/// this representation keeps the five camel stacks and the desert tiles in
/// fixed-size arrays. Cloning a board is then a few dozen bytes of stack copy,
/// and a move never allocates. The rules behaviour is identical to `move_unit`
/// (the equivalence tests at the bottom of this module pin that down).
#[derive(Clone)]
struct FastBoard {
    /// Camel indices (bottom to top) at each space; only `lengths[space]`
    /// entries are meaningful. Indices above `MAX_FIELD` hold finished units.
    stacks: [[u8; 5]; BOARD_SPACES],
    lengths: [u8; BOARD_SPACES],
    /// Current space of each camel index, for O(1) source lookup.
    positions: [u8; 5],
    deserts: [Option<DesertTile>; MAX_FIELD as usize + 1],
    finished: bool,
}

impl FastBoard {
    fn from_state(state: &State) -> Self {
        let mut board = Self {
            stacks: [[0; 5]; BOARD_SPACES],
            lengths: [0; BOARD_SPACES],
            positions: [0; 5],
            deserts: [None; MAX_FIELD as usize + 1],
            finished: state.is_finished(),
        };
        for (&space, cell) in &state.data {
            let index = space as usize;
            match cell {
                Space::Camels(camels) => {
                    for &camel in camels {
                        let ci = camel_index(camel);
                        let len = board.lengths[index] as usize;
                        board.stacks[index][len] = ci as u8;
                        board.lengths[index] = (len + 1) as u8;
                        board.positions[ci] = space;
                    }
                }
                Space::Desert(tile) => {
                    if index <= MAX_FIELD as usize {
                        board.deserts[index] = Some(*tile);
                    }
                }
            }
        }
        board
    }

    fn append(&mut self, space: usize, unit: &[u8]) {
        debug_assert!(space < BOARD_SPACES);
        let len = self.lengths[space] as usize;
        for (offset, &camel) in unit.iter().enumerate() {
            self.stacks[space][len + offset] = camel;
            self.positions[camel as usize] = space as u8;
        }
        self.lengths[space] = (len + unit.len()) as u8;
    }

    fn prepend(&mut self, space: usize, unit: &[u8]) {
        debug_assert!(space < BOARD_SPACES);
        let len = self.lengths[space] as usize;
        for index in (0..len).rev() {
            self.stacks[space][index + unit.len()] = self.stacks[space][index];
        }
        for (offset, &camel) in unit.iter().enumerate() {
            self.stacks[space][offset] = camel;
            self.positions[camel as usize] = space as u8;
        }
        self.lengths[space] = (len + unit.len()) as u8;
    }

    fn assert_no_adjacent_desert(&self, space: usize, neighbour: usize) {
        if self.deserts.get(neighbour).copied().flatten().is_some() {
            panic!(
                "precondition violated: desert at {} adjacent to desert at {}",
                space, neighbour
            );
        }
    }

    /// Move one camel unit. Returns the desert space when a tile was triggered.
    fn move_unit(&mut self, camel: Camel, steps: u8) -> Option<u8> {
        let ci = camel_index(camel);
        let source = self.positions[ci] as usize;
        let index_in_stack = (0..self.lengths[source] as usize)
            .find(|&index| self.stacks[source][index] as usize == ci)
            .expect("camel must be present in its recorded stack");
        let top = self.lengths[source] as usize;
        let unit_len = top - index_in_stack;
        let mut unit = [0u8; 5];
        unit[..unit_len].copy_from_slice(&self.stacks[source][index_in_stack..top]);
        self.lengths[source] = index_in_stack as u8;

        let destination = source as u8 + steps;
        let tile = if destination <= MAX_FIELD {
            self.deserts[destination as usize]
        } else {
            None
        };
        let oasis_at_destination = tile == Some(DesertTile::Oasis);
        let crosses_finish =
            destination > MAX_FIELD || (destination == MAX_FIELD && oasis_at_destination);

        if crosses_finish {
            let finish_space = if oasis_at_destination {
                destination as usize + 1
            } else {
                destination as usize
            };
            self.append(finish_space, &unit[..unit_len]);
            self.finished = true;
            return oasis_at_destination.then_some(destination);
        }

        let hit = tile.map(|_| destination);
        match tile {
            None => self.append(destination as usize, &unit[..unit_len]),
            Some(DesertTile::Oasis) => {
                let forward = destination as usize + 1;
                self.assert_no_adjacent_desert(destination as usize, forward);
                self.append(forward, &unit[..unit_len]);
            }
            Some(DesertTile::Mirage) => {
                let back = (destination as usize)
                    .checked_sub(1)
                    .expect("a mirage can never be reached from space 0");
                self.assert_no_adjacent_desert(destination as usize, back);
                self.prepend(back, &unit[..unit_len]);
            }
        }
        hit
    }

    /// Race order: most advanced space first, then stack top to bottom.
    fn ranking(&self) -> Vec<Camel> {
        let mut ranking = Vec::with_capacity(CAMEL_ORDER.len());
        for space in (0..BOARD_SPACES).rev() {
            let length = self.lengths[space] as usize;
            for index in (0..length).rev() {
                ranking.push(camel_from_index(self.stacks[space][index] as usize));
            }
        }
        ranking
    }
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
    /// A desert tile on 16 that was not triggered stays on the board. An oasis
    /// that was hit is reported as a desert hit and also stays until end of leg.
    ///
    /// Panics if:
    /// - the race has already finished
    /// - `steps < 1`
    /// - `camel` is not found in `data`
    /// - `camel` appears more than once in `data`
    /// - the move would overflow `u8`
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
    /// Each permutation/dice combination is one equally likely scenario. A
    /// scenario is simulated with [`State::move_multiple_units`], which stops as
    /// soon as a camel crosses the finish line; dice for camels later in that
    /// permutation are then never rolled. Those suffixes still count as distinct
    /// scenarios (the full set of permutations and dice combinations is the
    /// denominator), and every camel still receives a ranking in the finished
    /// state.
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
        let remaining = self.validate_simulation_input(already_moved);
        self.simulate_outcomes_fast(&remaining)
    }

    /// Check the preconditions shared by [`State::simulate_outcomes`] and the
    /// desert evaluators, and return the camels still to be moved.
    fn validate_simulation_input(&self, already_moved: &[Camel]) -> Vec<Camel> {
        if self.is_finished() {
            panic!("simulate_outcomes: the race has already finished");
        }

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
        for &camel in &CAMEL_ORDER {
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
        let remaining: Vec<Camel> = CAMEL_ORDER
            .iter()
            .copied()
            .filter(|camel| !already_moved.contains(camel))
            .collect();
        if remaining.is_empty() {
            panic!(
                "simulate_outcomes: all camels have already moved; the leg is over and there are no outcomes left to simulate"
            );
        }
        remaining
    }

    /// Exhaustive simulation over an allocation-light board. `remaining` is the
    /// validated list of camels whose dice are still in the pyramid.
    fn simulate_outcomes_fast(
        &self,
        remaining: &[Camel],
    ) -> (
        std::collections::BTreeMap<Camel, Vec<usize>>,
        std::collections::BTreeMap<u8, usize>,
    ) {
        use crate::combinatorics::{Permutations, Product};

        let base = FastBoard::from_state(self);
        let choices = vec![1u8, 2, 3];
        let mut counts: std::collections::BTreeMap<Camel, Vec<usize>> = CAMEL_ORDER
            .iter()
            .map(|&camel| (camel, vec![0usize; CAMEL_ORDER.len()]))
            .collect();
        let mut desert_counts: std::collections::BTreeMap<u8, usize> =
            std::collections::BTreeMap::new();

        for perm in Permutations::new(remaining.iter().copied()) {
            for dice in Product::new(choices.clone(), remaining.len()) {
                let mut board = base.clone();
                for (&camel, &steps) in perm.iter().zip(dice.iter()) {
                    if board.finished {
                        break;
                    }
                    if let Some(space) = board.move_unit(camel, steps) {
                        *desert_counts.entry(space).or_insert(0) += 1;
                    }
                }
                for (position, &camel) in board.ranking().iter().enumerate() {
                    counts.get_mut(&camel).expect("all camels are tracked")[position] += 1;
                }
            }
        }

        (counts, desert_counts)
    }

    /// Desert hit counts for a concrete move sequence, using the fast board.
    /// Used by the cockpit's sampled partial-board evaluator.
    pub(crate) fn simulate_moves_hits<I>(&self, moves: I) -> BTreeMap<u8, usize>
    where
        I: IntoIterator<Item = (Camel, u8)>,
    {
        let mut board = FastBoard::from_state(self);
        let mut counts = BTreeMap::new();
        for (camel, steps) in moves {
            if board.finished {
                break;
            }
            if let Some(space) = board.move_unit(camel, steps) {
                *counts.entry(space).or_insert(0) += 1;
            }
        }
        counts
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
        self.simulate_desert_hits(&[]).0
    }

    /// Expected direct income from placing a desert tile on each legal space
    /// for the rest of the current leg: 1 EP per camel unit that ends a move on
    /// the tile (RULES.md section 6).
    ///
    /// `already_moved` lists the camels whose dice have already been revealed;
    /// only the remaining moves are simulated. The denominator is the number of
    /// simulated permutations times die-roll products, matching
    /// [`State::simulate_outcomes`]. Tiles already on the board stay there, so
    /// their effects are included.
    ///
    /// Returns an empty map when the race is finished, the board has no camels,
    /// or the leg is over (all five dice revealed). As with
    /// [`State::simulate_outcomes`], all five camels must be on the board.
    pub fn expected_desert_income(
        &self,
        already_moved: &[Camel],
    ) -> std::collections::BTreeMap<u8, std::collections::BTreeMap<DesertTile, f64>> {
        let (hits, total) = self.simulate_desert_hits(already_moved);
        if total == 0 {
            return std::collections::BTreeMap::new();
        }
        hits.into_iter()
            .map(|(space, per_tile)| {
                (
                    space,
                    per_tile
                        .into_iter()
                        .map(|(tile, count)| (tile, count as f64 / total as f64))
                        .collect(),
                )
            })
            .collect()
    }

    /// Whether a tile may legally be placed on `space`: never space 1, never an
    /// occupied space, and never next to another desert (RULES.md section 6).
    fn is_legal_desert_placement(&self, space: u8) -> bool {
        if space == 1 || space > MAX_FIELD || self.data.contains_key(&space) {
            return false;
        }
        if space
            .checked_sub(1)
            .map(|left| matches!(self.data.get(&left), Some(Space::Desert(_))))
            .unwrap_or(false)
        {
            return false;
        }
        if space
            .checked_add(1)
            .map(|right| matches!(self.data.get(&right), Some(Space::Desert(_))))
            .unwrap_or(false)
        {
            return false;
        }
        true
    }

    /// Legal desert spaces, starting at the rearmost camel. Camel positions
    /// never decrease across a move, so spaces behind it cannot earn income.
    pub(crate) fn legal_desert_spaces(&self) -> Vec<u8> {
        if self.is_finished() || self.data.is_empty() {
            return Vec::new();
        }

        // Start from the rearmost camel, not the lowest occupied key: an
        // existing desert behind it is inert and must not widen the range.
        let Some(min_key) = self
            .data
            .iter()
            .filter_map(|(&key, space)| matches!(space, Space::Camels(_)).then_some(key))
            .min()
        else {
            return Vec::new();
        };

        (min_key..=MAX_FIELD)
            .filter(|&space| self.is_legal_desert_placement(space))
            .collect()
    }

    /// Simulate each legal placement, returning hit counts per space and tile
    /// together with the scenario count used as the expected-value denominator.
    ///
    /// Every candidate needs its own simulation pass: a tile changes where the
    /// camel that hits it ends up, and when that camel shares a stack with an
    /// unmoved camel it can change the latter's landing space too. Recording
    /// landings once on a tile-free board would therefore undercount hits such
    /// as a stacked camel returning to a mirage (`mirage_carries_a_stacked_camel_back_onto_the_tile`).
    fn simulate_desert_hits(
        &self,
        already_moved: &[Camel],
    ) -> (
        std::collections::BTreeMap<u8, std::collections::BTreeMap<DesertTile, usize>>,
        usize,
    ) {
        let mut result: std::collections::BTreeMap<
            u8,
            std::collections::BTreeMap<DesertTile, usize>,
        > = std::collections::BTreeMap::new();
        let mut total = 0usize;

        if already_moved.len() >= CAMEL_ORDER.len() {
            return (result, total);
        }

        let candidates: Vec<(u8, DesertTile)> = self
            .legal_desert_spaces()
            .into_iter()
            .flat_map(|space| [(space, DesertTile::Oasis), (space, DesertTile::Mirage)])
            .collect();
        if candidates.is_empty() {
            return (result, total);
        }

        // The candidate boards are independent, so spread them over OS threads.
        // Each thread reuses the same permutation/dice enumeration for its
        // candidates and never allocates during a scenario.
        let remaining = self.validate_simulation_input(already_moved);
        let remaining_ref = &remaining;
        let threads = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
            .min(candidates.len())
            .max(1);
        let chunk_size = candidates.len().div_ceil(threads);

        let evaluated: Vec<(u8, DesertTile, usize, usize)> = std::thread::scope(|scope| {
            let handles: Vec<_> = candidates
                .chunks(chunk_size)
                .map(|chunk| {
                    scope.spawn(move || {
                        let mut evaluated = Vec::with_capacity(chunk.len());
                        for &(space, tile) in chunk {
                            let mut board = self.data.clone();
                            board.insert(space, Space::Desert(tile));
                            let candidate = State::new(board);
                            let (camel_counts, desert_counts) =
                                candidate.simulate_outcomes_fast(remaining_ref);
                            let hits = desert_counts.get(&space).copied().unwrap_or(0usize);
                            let scenarios = camel_counts
                                .values()
                                .next()
                                .map(|positions| positions.iter().sum())
                                .unwrap_or(0);
                            evaluated.push((space, tile, hits, scenarios));
                        }
                        evaluated
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|handle| match handle.join() {
                    Ok(values) => values,
                    Err(panic) => std::panic::resume_unwind(panic),
                })
                .collect()
        });

        for (space, tile, hits, scenarios) in evaluated {
            if total == 0 {
                total = scenarios;
            }
            result.entry(space).or_default().insert(tile, hits);
        }

        (result, total)
    }
}

#[cfg(test)]
mod fast_simulator_tests {
    use super::*;

    /// The pre-optimisation implementation, kept as an oracle for the
    /// allocation-light simulator. It uses `move_multiple_units` and a fresh
    /// full `State` for every scenario, exactly as the old `simulate_outcomes`
    /// did.
    fn reference_outcomes(
        state: &State,
        already_moved: &[Camel],
    ) -> (BTreeMap<Camel, Vec<usize>>, BTreeMap<u8, usize>) {
        use crate::combinatorics::{Permutations, Product};

        let remaining: Vec<Camel> = CAMEL_ORDER
            .iter()
            .copied()
            .filter(|camel| !already_moved.contains(camel))
            .collect();
        let choices = vec![1u8, 2, 3];
        let mut counts: BTreeMap<Camel, Vec<usize>> = CAMEL_ORDER
            .iter()
            .map(|&camel| (camel, vec![0usize; CAMEL_ORDER.len()]))
            .collect();
        let mut desert_counts = BTreeMap::new();

        for perm in Permutations::new(remaining.clone()) {
            for dice in Product::new(choices.clone(), remaining.len()) {
                let (end, hits) =
                    state.move_multiple_units(perm.iter().copied().zip(dice.iter().copied()));
                for (position, &camel) in end.ranking().iter().enumerate() {
                    counts.get_mut(&camel).unwrap()[position] += 1;
                }
                for (space, count) in hits {
                    *desert_counts.entry(space).or_insert(0) += count;
                }
            }
        }
        (counts, desert_counts)
    }

    fn reference_expected_income(
        state: &State,
        already_moved: &[Camel],
    ) -> BTreeMap<u8, BTreeMap<DesertTile, f64>> {
        let mut hits: BTreeMap<u8, BTreeMap<DesertTile, usize>> = BTreeMap::new();
        let mut total = 0usize;
        for space in state.legal_desert_spaces() {
            for &tile in &[DesertTile::Oasis, DesertTile::Mirage] {
                let mut board = state.data.clone();
                board.insert(space, Space::Desert(tile));
                let (counts, desert_counts) = reference_outcomes(&State::new(board), already_moved);
                if total == 0 {
                    total = counts.values().next().map(|v| v.iter().sum()).unwrap_or(0);
                }
                let count = desert_counts.get(&space).copied().unwrap_or(0);
                hits.entry(space).or_default().insert(tile, count);
            }
        }
        if total == 0 {
            return BTreeMap::new();
        }
        hits.into_iter()
            .map(|(space, per_tile)| {
                (
                    space,
                    per_tile
                        .into_iter()
                        .map(|(tile, count)| (tile, count as f64 / total as f64))
                        .collect(),
                )
            })
            .collect()
    }

    fn state(stacks: &[(u8, Vec<Camel>)], deserts: &[(u8, DesertTile)]) -> State {
        let mut data = BTreeMap::new();
        for &(space, ref camels) in stacks {
            data.insert(space, Space::Camels(camels.clone()));
        }
        for &(space, tile) in deserts {
            data.insert(space, Space::Desert(tile));
        }
        State::new(data)
    }

    fn simple_board() -> State {
        state(
            &[
                (2, vec![Camel::Yellow]),
                (3, vec![Camel::Orange]),
                (5, vec![Camel::White]),
                (7, vec![Camel::Green]),
                (9, vec![Camel::Blue]),
            ],
            &[],
        )
    }

    /// Stacks matter: a lower camel carries the ones above it, and a desert can
    /// move the whole unit before the upper camel's own die is revealed.
    fn stacked_board() -> State {
        state(
            &[
                (5, vec![Camel::White, Camel::Yellow, Camel::Orange]),
                (8, vec![Camel::Green]),
                (11, vec![Camel::Blue]),
            ],
            &[(6, DesertTile::Oasis), (10, DesertTile::Mirage)],
        )
    }

    fn near_finish_board() -> State {
        state(
            &[
                (12, vec![Camel::White]),
                (13, vec![Camel::Yellow]),
                (14, vec![Camel::Orange, Camel::Green]),
                (15, vec![Camel::Blue]),
            ],
            &[(16, DesertTile::Oasis)],
        )
    }

    #[test]
    fn fast_outcomes_match_reference_on_simple_board() {
        let board = simple_board();
        assert_eq!(
            board.simulate_outcomes(&[]).0,
            reference_outcomes(&board, &[]).0
        );
        assert_eq!(
            board.simulate_outcomes(&[]).1,
            reference_outcomes(&board, &[]).1
        );
    }

    #[test]
    fn fast_outcomes_match_reference_with_stacks_and_deserts() {
        let board = stacked_board();
        let expected = reference_outcomes(&board, &[]);
        let actual = board.simulate_outcomes(&[]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn fast_outcomes_match_reference_with_partial_legs() {
        let board = stacked_board();
        for already_moved in [
            vec![Camel::White],
            vec![Camel::White, Camel::Yellow],
            vec![Camel::Blue, Camel::Green],
        ] {
            assert_eq!(
                board.simulate_outcomes(&already_moved),
                reference_outcomes(&board, &already_moved),
                "already_moved = {already_moved:?}"
            );
        }
    }

    #[test]
    fn fast_outcomes_match_reference_near_the_finish() {
        let board = near_finish_board();
        assert_eq!(
            board.simulate_outcomes(&[]),
            reference_outcomes(&board, &[])
        );
    }

    /// A mirage can push a stacked unit backwards so its upper camel lands on
    /// the same tile twice. A simplified "one landing per camel" shortcut gets
    /// this wrong; the full scenario simulation must not.
    fn mirage_board() -> State {
        state(
            &[
                (2, vec![Camel::Orange]),
                (5, vec![Camel::White, Camel::Yellow]),
                (9, vec![Camel::Green]),
                (12, vec![Camel::Blue]),
            ],
            &[(7, DesertTile::Mirage)],
        )
    }

    #[test]
    fn mirage_carries_a_stacked_camel_back_onto_the_tile() {
        let board = mirage_board();
        assert_eq!(
            board.simulate_outcomes(&[]),
            reference_outcomes(&board, &[])
        );
    }

    #[test]
    fn parallel_desert_options_match_the_reference_on_stacked_boards() {
        // Two dice left keeps the oracle cheap while still exercising every
        // candidate, including the oasis on 16 in the near-finish board.
        let already_moved = [Camel::White, Camel::Yellow, Camel::Orange];
        for board in [
            simple_board(),
            stacked_board(),
            near_finish_board(),
            mirage_board(),
        ] {
            assert_eq!(
                board.expected_desert_income(&already_moved),
                reference_expected_income(&board, &already_moved),
            );
        }
    }

    #[test]
    fn fast_move_hits_match_move_multiple_units() {
        let board = stacked_board();
        let moves = [
            (Camel::White, 2u8),
            (Camel::Yellow, 1),
            (Camel::Green, 3),
            (Camel::Blue, 2),
            (Camel::Orange, 1),
        ];
        let (_, expected) = board.move_multiple_units(moves);
        assert_eq!(board.simulate_moves_hits(moves), expected);
    }

    #[test]
    fn fast_move_hits_count_multiple_landings_on_one_tile() {
        let board = mirage_board();
        let moves = [(Camel::White, 2u8), (Camel::Yellow, 1u8)];
        assert_eq!(
            board.simulate_moves_hits(moves),
            BTreeMap::from([(7, 2usize)])
        );
    }
}
