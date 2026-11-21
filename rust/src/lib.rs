pub mod combinatorics;
pub mod simulation;
pub mod state;

pub use combinatorics::{Permutations, Product};
pub use simulation::{
    simulate_overall_winner_loser, simulate_overall_winner_loser_with, OverallOdds,
};
// Public API renamed to better reflect game terminology (breaking change).
// `MAX_FIELD` is the highest track space (16). Camel units may be stored at
// higher keys to represent having crossed the finish line; desert tiles are
// always confined to spaces 1..=16.
pub use state::{Camel, DesertTile, Space, State, MAX_FIELD};
