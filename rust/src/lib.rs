pub mod combinatorics;
pub mod simulation;
pub mod state;

pub use combinatorics::{Permutations, Product};
pub use simulation::{
    simulate_overall_winner_loser, simulate_overall_winner_loser_with, OverallOdds,
};
// Public API renamed to better reflect game terminology (breaking change).
// `State` also has an internal `finished` field: construct it with `State::new(data)`
// rather than a `State { data }` struct literal.
pub use state::{Camel, DesertTile, Space, State, MAX_FIELD};
