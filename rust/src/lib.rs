pub mod combinatorics;
pub mod state;

pub use combinatorics::{Permutations, Product};
// Public API renamed to better reflect game terminology (breaking change).
pub use state::{Camel, DesertTile, Space, State};
