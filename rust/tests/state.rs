use camel_up::{Camel, Space, State};
use std::collections::BTreeMap;

fn mk_map(entries: &[(u8, Vec<Camel>)]) -> BTreeMap<u8, Space> {
    let mut m = BTreeMap::new();
    for (k, v) in entries {
        m.insert(*k, Space::Camels(v.clone()));
    }
    m
}

#[test]
fn test_move_white_combinations() {
    let field = 1;
    let steps = 1;

    // Four possibilities for data[field]
    let v1 = vec![Camel::White];
    let v2 = vec![Camel::White, Camel::Yellow];
    let v3 = vec![Camel::Yellow, Camel::White, Camel::Orange];
    let v4 = vec![Camel::Yellow, Camel::White];

    // Two possibilities for data[field + 1]
    let nv_green = vec![Camel::Green];

    // Helper to run a single scenario: initial map entries, expected map entries
    let run = |initial_entries: &[(u8, Vec<Camel>)], expected_entries: &[(u8, Vec<Camel>)]| {
        let state = State::new(mk_map(initial_entries));
        let (res_state, desert_hit) = state.move_unit(Camel::White, steps);
        assert_eq!(desert_hit, None);
        let expected = mk_map(expected_entries);
        assert_eq!(res_state.data, expected);
    };

    // 1) v1, nv empty -> expected {2: [WHITE]}
    run(&[(field, v1.clone())], &[(field + 1, vec![Camel::White])]);

    // 2) v1, nv = [GREEN] -> expected {2: [GREEN, WHITE]}
    run(
        &[(field, v1.clone()), (field + 1, nv_green.clone())],
        &[(field + 1, vec![Camel::Green, Camel::White])],
    );

    // 3) v2 ([WHITE, YELLOW]), nv empty -> expected {2: [WHITE, YELLOW]}
    run(
        &[(field, v2.clone())],
        &[(field + 1, vec![Camel::White, Camel::Yellow])],
    );

    // 4) v2, nv = [GREEN] -> expected {2: [GREEN, WHITE, YELLOW]}
    run(
        &[(field, v2.clone()), (field + 1, nv_green.clone())],
        &[(field + 1, vec![Camel::Green, Camel::White, Camel::Yellow])],
    );

    // 5) v3 ([YELLOW, WHITE, ORANGE]), nv empty -> expected {1: [YELLOW], 2: [WHITE, ORANGE]}
    run(
        &[(field, v3.clone())],
        &[
            (field, vec![Camel::Yellow]),
            (field + 1, vec![Camel::White, Camel::Orange]),
        ],
    );

    // 6) v3, nv = [GREEN] -> expected {1: [YELLOW], 2: [GREEN, WHITE, ORANGE]}
    run(
        &[(field, v3.clone()), (field + 1, nv_green.clone())],
        &[
            (field, vec![Camel::Yellow]),
            (field + 1, vec![Camel::Green, Camel::White, Camel::Orange]),
        ],
    );

    // 7) v4 ([YELLOW, WHITE]), nv empty -> expected {1: [YELLOW], 2: [WHITE]}
    run(
        &[(field, v4.clone())],
        &[
            (field, vec![Camel::Yellow]),
            (field + 1, vec![Camel::White]),
        ],
    );

    // 8) v4, nv = [GREEN] -> expected {1: [YELLOW], 2: [GREEN, WHITE]}
    run(
        &[(field, v4.clone()), (field + 1, nv_green.clone())],
        &[
            (field, vec![Camel::Yellow]),
            (field + 1, vec![Camel::Green, Camel::White]),
        ],
    );
}

#[test]
fn test_move_all_camels_various_steps() {
    // initial map: 1:[WHITE], 3:[YELLOW, ORANGE, GREEN], 5:[BLUE]
    let initial = vec![
        (1, vec![Camel::White]),
        (3, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
        (5, vec![Camel::Blue]),
    ];
    let state = State::new(mk_map(&initial));

    // Helper to assert expected map
    let expect = |res_state: State, expected_entries: &[(u8, Vec<Camel>)]| {
        assert_eq!(res_state.data, mk_map(expected_entries));
    };

    // WHITE at 1:
    {
        let (res, dh) = state.move_unit(Camel::White, 1);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (2, vec![Camel::White]),
                (3, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                (5, vec![Camel::Blue]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::White, 2);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (
                    3,
                    vec![Camel::Yellow, Camel::Orange, Camel::Green, Camel::White],
                ),
                (5, vec![Camel::Blue]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::White, 3);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (3, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                (4, vec![Camel::White]),
                (5, vec![Camel::Blue]),
            ],
        );
    }

    // YELLOW at 3 (pos 0)
    {
        let (res, dh) = state.move_unit(Camel::Yellow, 1);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (4, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                (5, vec![Camel::Blue]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Yellow, 2);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (
                    5,
                    vec![Camel::Blue, Camel::Yellow, Camel::Orange, Camel::Green],
                ),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Yellow, 3);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (5, vec![Camel::Blue]),
                (6, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
            ],
        );
    }

    // ORANGE at 3 (pos 1)
    {
        let (res, dh) = state.move_unit(Camel::Orange, 1);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow]),
                (4, vec![Camel::Orange, Camel::Green]),
                (5, vec![Camel::Blue]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Orange, 2);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow]),
                (5, vec![Camel::Blue, Camel::Orange, Camel::Green]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Orange, 3);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow]),
                (5, vec![Camel::Blue]),
                (6, vec![Camel::Orange, Camel::Green]),
            ],
        );
    }

    // GREEN at 3 (pos 2)
    {
        let (res, dh) = state.move_unit(Camel::Green, 1);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow, Camel::Orange]),
                (4, vec![Camel::Green]),
                (5, vec![Camel::Blue]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Green, 2);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow, Camel::Orange]),
                (5, vec![Camel::Blue, Camel::Green]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Green, 3);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow, Camel::Orange]),
                (5, vec![Camel::Blue]),
                (6, vec![Camel::Green]),
            ],
        );
    }

    // BLUE at 5 (pos 0)
    {
        let (res, dh) = state.move_unit(Camel::Blue, 1);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                (6, vec![Camel::Blue]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Blue, 2);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                (7, vec![Camel::Blue]),
            ],
        );
    }
    {
        let (res, dh) = state.move_unit(Camel::Blue, 3);
        assert_eq!(dh, None);
        expect(
            res,
            &[
                (1, vec![Camel::White]),
                (3, vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                (8, vec![Camel::Blue]),
            ],
        );
    }
}

#[test]
fn landing_exactly_on_16_does_not_finish_the_race() {
    let state = State::new(mk_map(&[(15, vec![Camel::White])]));

    let (res, desert_hit) = state.move_unit(Camel::White, 1);

    assert_eq!(desert_hit, None);
    assert_eq!(res.data, mk_map(&[(16, vec![Camel::White])]));
    assert!(!res.is_finished());
}

#[test]
fn overshooting_16_finishes_the_race_with_crossing_unit_first() {
    // White at 15 moves 3 (would land on 18); Blue is already on 16.
    let state = State::new(mk_map(&[(15, vec![Camel::White]), (16, vec![Camel::Blue])]));

    let (res, desert_hit) = state.move_unit(Camel::White, 3);

    assert_eq!(desert_hit, None);
    // White leaves the track at 18; Blue stays on space 16.
    assert_eq!(
        res.data,
        mk_map(&[(16, vec![Camel::Blue]), (18, vec![Camel::White])])
    );
    assert!(res.is_finished());
    assert_eq!(res.ranking(), vec![Camel::White, Camel::Blue]);
}

#[test]
fn moving_from_16_finishes_the_race_and_keeps_stack_order() {
    let state = State::new(mk_map(&[(16, vec![Camel::White, Camel::Yellow])]));

    let (res, _) = state.move_unit(Camel::White, 1);

    assert_eq!(res.data, mk_map(&[(17, vec![Camel::White, Camel::Yellow])]));
    assert!(res.is_finished());
    assert_eq!(res.ranking(), vec![Camel::Yellow, Camel::White]);

    // Moving the top camel instead leaves the lower one underneath it.
    let (res2, _) = state.move_unit(Camel::Yellow, 2);
    assert_eq!(
        res2.data,
        mk_map(&[(16, vec![Camel::White]), (18, vec![Camel::Yellow])])
    );
    assert!(res2.is_finished());
}

#[test]
fn oasis_on_16_finishes_the_race_and_reports_the_hit() {
    use camel_up::{DesertTile, Space};

    let mut map = mk_map(&[(14, vec![Camel::White]), (10, vec![Camel::Blue])]);
    map.insert(16, Space::Desert(DesertTile::Oasis));
    let state = State::new(map);

    let (res, desert_hit) = state.move_unit(Camel::White, 2);

    assert_eq!(desert_hit, Some(16));
    let mut expected = mk_map(&[(10, vec![Camel::Blue]), (17, vec![Camel::White])]);
    expected.insert(16, Space::Desert(DesertTile::Oasis));
    assert_eq!(res.data, expected);
    assert!(res.is_finished());
    assert_eq!(res.ranking().first(), Some(&Camel::White));
}

#[test]
fn mirage_on_16_does_not_finish_the_race() {
    use camel_up::{DesertTile, Space};

    let mut map = mk_map(&[(14, vec![Camel::White])]);
    map.insert(16, Space::Desert(DesertTile::Mirage));
    let state = State::new(map);

    let (res, desert_hit) = state.move_unit(Camel::White, 2);

    assert_eq!(desert_hit, Some(16));
    assert_eq!(res.data.get(&15), Some(&Space::Camels(vec![Camel::White])));
    assert!(!res.is_finished());
}

#[test]
fn move_multiple_units_stops_after_the_finish_line() {
    let state = State::new(mk_map(&[(15, vec![Camel::White]), (16, vec![Camel::Blue])]));

    let (res, desert_hits) = state.move_multiple_units([(Camel::White, 3), (Camel::Blue, 1)]);

    assert_eq!(
        res.data,
        mk_map(&[(16, vec![Camel::Blue]), (18, vec![Camel::White])])
    );
    assert!(res.is_finished());
    assert!(desert_hits.is_empty());
}

#[test]
#[should_panic(expected = "after the race has finished")]
fn moving_in_a_finished_state_panics() {
    let state = State::new(mk_map(&[(16, vec![Camel::White])]));
    let (finished, _) = state.move_unit(Camel::White, 1);

    finished.move_unit(Camel::White, 1);
}

#[test]
fn oasis_on_16_carries_the_whole_stack_over_the_finish() {
    use camel_up::{DesertTile, Space};

    let mut map = mk_map(&[
        (14, vec![Camel::White, Camel::Yellow]),
        (10, vec![Camel::Blue]),
    ]);
    map.insert(16, Space::Desert(DesertTile::Oasis));
    let state = State::new(map);

    let (res, desert_hit) = state.move_unit(Camel::White, 2);

    assert_eq!(desert_hit, Some(16));
    assert_eq!(
        res.data.get(&17),
        Some(&Space::Camels(vec![Camel::White, Camel::Yellow]))
    );
    assert_eq!(res.data.get(&16), Some(&Space::Desert(DesertTile::Oasis)));
    assert!(res.is_finished());
    // Yellow stays on top of the carried unit, so it is 1st at the finish.
    assert_eq!(
        res.ranking(),
        vec![Camel::Yellow, Camel::White, Camel::Blue]
    );
}

#[test]
fn mirage_on_16_is_not_triggered_by_a_dice_overshoot() {
    use camel_up::{DesertTile, Space};

    let mut map = mk_map(&[(15, vec![Camel::White])]);
    map.insert(16, Space::Desert(DesertTile::Mirage));
    let state = State::new(map);

    let (res, desert_hit) = state.move_unit(Camel::White, 3);

    assert_eq!(desert_hit, None);
    assert_eq!(res.data.get(&18), Some(&Space::Camels(vec![Camel::White])));
    // The dice move jumped over the tile, so it was never triggered or removed.
    assert_eq!(res.data.get(&16), Some(&Space::Desert(DesertTile::Mirage)));
    assert!(res.is_finished());
}

#[test]
fn finished_state_has_no_desert_placements_to_evaluate() {
    let state = State::new(mk_map(&[(16, vec![Camel::White])]));
    let (finished, _) = state.move_unit(Camel::White, 1);

    assert!(finished.is_finished());
    assert!(finished.evaluate_desert_placements().is_empty());
}

#[test]
fn finished_state_survives_reconstruction_from_data() {
    let state = State::new(mk_map(&[(16, vec![Camel::White])]));
    let (finished, _) = state.move_unit(Camel::White, 1);

    let rebuilt = State::new(finished.data.clone());
    assert!(rebuilt.is_finished());
    assert_eq!(rebuilt.data, finished.data);
}

#[test]
fn only_camels_beyond_16_signal_a_finished_race() {
    use camel_up::{DesertTile, Space};

    let camel_finished = State::new(mk_map(&[(17, vec![Camel::White])]));
    assert!(camel_finished.is_finished());

    // A desert outside the track is not a legal state, but it must not be
    // mistaken for a camel that has crossed the finish line.
    let mut map = mk_map(&[(14, vec![Camel::White])]);
    map.insert(17, Space::Desert(DesertTile::Mirage));
    let state = State::new(map);
    assert!(!state.is_finished());
}
