use camel_cup::{Camel, DesertTile, Field, State};
use std::collections::BTreeMap;
use std::time::Instant;

fn mk_map(entries: &[(u8, Field)]) -> BTreeMap<u8, Field> {
    let mut m = BTreeMap::new();
    for (k, v) in entries {
        m.insert(*k, v.clone());
    }
    m
}

fn main() {
    // Initial position: {1: [Green], 2: [Yellow, White], 3: [Blue, Orange]}, no deserts
    let initial = vec![
        (1u8, Field::Camels(vec![Camel::Green])),
        (2u8, Field::Camels(vec![Camel::Yellow, Camel::White])),
        (3u8, Field::Camels(vec![Camel::Blue, Camel::Orange])),
        (4u8, Field::Desert(DesertTile::Mirage)),
    ];
    let state = State::new(mk_map(&initial));

    // Run simulate_outcomes
    let t0 = Instant::now();
    let (camel_counts, desert_counts) = state.simulate_outcomes();
    let elapsed_sim = t0.elapsed();
    println!(
        "simulate_outcomes runtime: {:.3}s",
        elapsed_sim.as_secs_f64()
    );

    println!("\nCamel counts (Camel -> Vec<usize>):");
    for (camel, vec) in camel_counts.iter() {
        println!("  {:?} => {:?}", camel, vec);
    }

    println!("\nDesert hit counts (field -> total hits):");
    for (pos, hits) in desert_counts.iter() {
        println!("  {} => {}", pos, hits);
    }

    // Run evaluate_desert_placements
    let t1 = Instant::now();
    let placements = state.evaluate_desert_placements();
    let elapsed_eval = t1.elapsed();
    println!(
        "\nevaluate_desert_placements runtime: {:.3}s",
        elapsed_eval.as_secs_f64()
    );

    println!("\nDesert placement impact (pos -> {{tile: hits}}):");
    for (pos, tile_map) in placements.iter() {
        println!("pos {}:", pos);
        for (tile, hits) in tile_map.iter() {
            println!("  {:?} => {}", tile, hits);
        }
    }
}
