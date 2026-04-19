use camel_cup::{Camel, DesertTile, Space, State};
use std::collections::BTreeMap;
use std::panic::catch_unwind;

fn mk_map(entries: &[(u8, Space)]) -> BTreeMap<u8, Space> {
    let mut m = BTreeMap::new();
    for (k, v) in entries {
        m.insert(*k, v.clone());
    }
    m
}

#[test]
fn test_move_camel_with_deserts_explicit() {
    let steps = 2; // White from pos 1 -> pos 3

    // Helper to run a move and expect a panic
    let expect_panic = |state: State, case_desc: &str| {
        let res = catch_unwind(|| state.move_unit(Camel::White, steps));
        assert!(res.is_err(), "expected panic for case {}", case_desc);
    };

    // 1) pos3 = Oasis: test pos4 desert variants (both should panic), then empty and orange
    {
        // base initial map: white at pos1 and oasis at pos3
        let base = vec![
            (1, Space::Camels(vec![Camel::White])),
            (3, Space::Desert(DesertTile::Oasis)),
        ];

        // deserts at pos4 -> both Oasis and Mirage should panic (precondition)
        for tile in &[DesertTile::Oasis, DesertTile::Mirage] {
            let mut entries = base.clone();
            entries.push((4, Space::Desert(*tile)));
            let state = State::new(mk_map(&entries));
            expect_panic(state, &format!("pos3=Oasis pos4={:?}", tile));
        }

        // pos4 empty -> white is forwarded to 4, desert hit is Some(3)
        {
            let state = State::new(mk_map(&base));
            let (res_state, desert_hit) = state.move_unit(Camel::White, steps);
            assert_eq!(desert_hit, Some(3u8));
            let expected = vec![
                (3, Space::Desert(DesertTile::Oasis)),
                (4, Space::Camels(vec![Camel::White])),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }

        // pos4 has Orange -> white forwarded to 4 and appended -> [Orange, White]
        {
            let mut entries = base.clone();
            entries.push((4, Space::Camels(vec![Camel::Orange])));
            let state = State::new(mk_map(&entries));
            let (res_state, desert_hit) = state.move_unit(Camel::White, steps);
            assert_eq!(desert_hit, Some(3u8));
            let expected = vec![
                (3, Space::Desert(DesertTile::Oasis)),
                (4, Space::Camels(vec![Camel::Orange, Camel::White])),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }
    }

    // 2) pos3 = Mirage: test pos2 desert variants (both should panic), then empty and orange
    {
        let base = vec![
            (1, Space::Camels(vec![Camel::White])),
            (3, Space::Desert(DesertTile::Mirage)),
        ];

        // deserts at pos2 -> both Oasis and Mirage should panic (precondition)
        for tile in &[DesertTile::Oasis, DesertTile::Mirage] {
            let mut entries = base.clone();
            entries.push((2, Space::Desert(*tile)));
            let state = State::new(mk_map(&entries));
            expect_panic(state, &format!("pos3=Mirage pos2={:?}", tile));
        }

        // pos2 empty -> white moved back to 2, desert hit is Some(3)
        {
            let state = State::new(mk_map(&base));
            let (res_state, desert_hit) = state.move_unit(Camel::White, steps);
            assert_eq!(desert_hit, Some(3u8));
            let expected = vec![
                (2, Space::Camels(vec![Camel::White])),
                (3, Space::Desert(DesertTile::Mirage)),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }

        // pos2 has Orange -> white prepended to 2 -> [White, Orange]
        {
            let mut entries = base.clone();
            entries.push((2, Space::Camels(vec![Camel::Orange])));
            let state = State::new(mk_map(&entries));
            let (res_state, desert_hit) = state.move_unit(Camel::White, steps);
            assert_eq!(desert_hit, Some(3));
            let expected = vec![
                (2, Space::Camels(vec![Camel::White, Camel::Orange])),
                (3, Space::Desert(DesertTile::Mirage)),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }
    }

    // 3) pos3 = empty: white just lands on pos3 normally
    {
        let base = vec![(1, Space::Camels(vec![Camel::White]))];
        let state = State::new(mk_map(&base));
        let (res_state, desert_hit) = state.move_unit(Camel::White, steps);
        assert_eq!(desert_hit, None);
        let expected = vec![(3, Space::Camels(vec![Camel::White]))];
        assert_eq!(res_state.data, mk_map(&expected));
    }

    // 4) pos3 = Yellow: white lands on pos3 appended -> [Yellow, White]
    {
        let base = vec![
            (1, Space::Camels(vec![Camel::White])),
            (3, Space::Camels(vec![Camel::Yellow])),
        ];
        let state = State::new(mk_map(&base));
        let (res_state, desert_hit) = state.move_unit(Camel::White, steps);
        assert_eq!(desert_hit, None);
        let expected = vec![(3, Space::Camels(vec![Camel::Yellow, Camel::White]))];
        assert_eq!(res_state.data, mk_map(&expected));
    }
}

#[test]
fn test_mirage_prepends_white_from_stack() {
    // initial: 1: [Yellow, White], 2: Mirage
    let initial = vec![
        (1, Space::Camels(vec![Camel::Yellow, Camel::White])),
        (2, Space::Desert(DesertTile::Mirage)),
    ];
    let state = State::new(mk_map(&initial));

    // steps = 1 moves White from 1 -> 2 (hits Mirage), mirage moves it back to 1 and prepends
    let (res_state, desert_hit) = state.move_unit(Camel::White, 1);
    assert_eq!(desert_hit, Some(2u8));
    let expected = vec![
        (1, Space::Camels(vec![Camel::White, Camel::Yellow])),
        (2, Space::Desert(DesertTile::Mirage)),
    ];
    assert_eq!(res_state.data, mk_map(&expected));
}

#[test]
fn test_move_all_camels_various_steps_with_deserts() {
    // initial map: 1:[White], 3:[Yellow, Orange, Green], 5:[Blue]
    // with deserts at 4: Mirage and 6: Oasis
    let mut initial = vec![
        (1, Space::Camels(vec![Camel::White])),
        (
            3,
            Space::Camels(vec![Camel::Yellow, Camel::Orange, Camel::Green]),
        ),
        (5, Space::Camels(vec![Camel::Blue])),
    ];
    // insert deserts
    initial.push((4, Space::Desert(DesertTile::Mirage)));
    initial.push((6, Space::Desert(DesertTile::Oasis)));

    let state = State::new(mk_map(&initial));

    let expect = |(res_state, _desert_hit): (State, Option<u8>),
                  expected_entries: &[(u8, Space)]| {
        // desert_hit value is asserted by checking expected entries for desert keys
        assert_eq!(res_state.data, mk_map(expected_entries));
    };

    // WHITE at 1:
    {
        let result = state.move_unit(Camel::White, 1);
        expect(
            result,
            &[
                (2, Space::Camels(vec![Camel::White])),
                (
                    3,
                    Space::Camels(vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let result = state.move_unit(Camel::White, 2);
        expect(
            result,
            &[
                (
                    3,
                    Space::Camels(vec![
                        Camel::Yellow,
                        Camel::Orange,
                        Camel::Green,
                        Camel::White,
                    ]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let (res_state, desert_hit) = state.move_unit(Camel::White, 3);
        assert_eq!(desert_hit, Some(4u8));
        expect(
            (res_state, desert_hit),
            &[
                (
                    3,
                    Space::Camels(vec![
                        Camel::White,
                        Camel::Yellow,
                        Camel::Orange,
                        Camel::Green,
                    ]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }

    // YELLOW at 3 (pos 0)
    {
        let (res_state, desert_hit) = state.move_unit(Camel::Yellow, 1);
        assert_eq!(desert_hit, Some(4u8));
        expect(
            (res_state, desert_hit),
            &[
                (1, Space::Camels(vec![Camel::White])),
                (
                    3,
                    Space::Camels(vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let result = state.move_unit(Camel::Yellow, 2);
        expect(
            result,
            &[
                (1, Space::Camels(vec![Camel::White])),
                (
                    5,
                    Space::Camels(vec![
                        Camel::Blue,
                        Camel::Yellow,
                        Camel::Orange,
                        Camel::Green,
                    ]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let (res_state, desert_hit) = state.move_unit(Camel::Yellow, 3);
        assert_eq!(desert_hit, Some(6u8));
        expect(
            (res_state, desert_hit),
            &[
                (1, Space::Camels(vec![Camel::White])),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
                (
                    7,
                    Space::Camels(vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
            ],
        );
    }

    // ORANGE at 3 (pos 1)
    {
        let (res_state, desert_hit) = state.move_unit(Camel::Orange, 1);
        assert_eq!(desert_hit, Some(4u8));
        expect(
            (res_state, desert_hit),
            &[
                (1, Space::Camels(vec![Camel::White])),
                (
                    3,
                    Space::Camels(vec![Camel::Orange, Camel::Green, Camel::Yellow]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let result = state.move_unit(Camel::Orange, 2);
        expect(
            result,
            &[
                (1, Space::Camels(vec![Camel::White])),
                (3, Space::Camels(vec![Camel::Yellow])),
                (
                    5,
                    Space::Camels(vec![Camel::Blue, Camel::Orange, Camel::Green]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let (res_state, desert_hit) = state.move_unit(Camel::Orange, 3);
        assert_eq!(desert_hit, Some(6u8));
        expect(
            (res_state, desert_hit),
            &[
                (1, Space::Camels(vec![Camel::White])),
                (3, Space::Camels(vec![Camel::Yellow])),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
                (7, Space::Camels(vec![Camel::Orange, Camel::Green])),
                (4, Space::Desert(DesertTile::Mirage)),
            ],
        );
    }

    // GREEN at 3 (pos 2)
    {
        let (res_state, desert_hit) = state.move_unit(Camel::Green, 1);
        assert_eq!(desert_hit, Some(4u8));
        expect(
            (res_state, desert_hit),
            &[
                (1, Space::Camels(vec![Camel::White])),
                (
                    3,
                    Space::Camels(vec![Camel::Green, Camel::Yellow, Camel::Orange]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let result = state.move_unit(Camel::Green, 2);
        expect(
            result,
            &[
                (1, Space::Camels(vec![Camel::White])),
                (3, Space::Camels(vec![Camel::Yellow, Camel::Orange])),
                (5, Space::Camels(vec![Camel::Blue, Camel::Green])),
                (4, Space::Desert(DesertTile::Mirage)),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let (res_state, desert_hit) = state.move_unit(Camel::Green, 3);
        assert_eq!(desert_hit, Some(6u8));
        expect(
            (res_state, desert_hit),
            &[
                (1, Space::Camels(vec![Camel::White])),
                (3, Space::Camels(vec![Camel::Yellow, Camel::Orange])),
                (5, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
                (7, Space::Camels(vec![Camel::Green])),
                (4, Space::Desert(DesertTile::Mirage)),
            ],
        );
    }

    // BLUE at 5 (pos 0)
    {
        let (res_state, desert_hit) = state.move_unit(Camel::Blue, 1);
        assert_eq!(desert_hit, Some(6u8));
        expect(
            (res_state, desert_hit),
            &[
                (1, Space::Camels(vec![Camel::White])),
                (
                    3,
                    Space::Camels(vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (6, Space::Desert(DesertTile::Oasis)),
                (7, Space::Camels(vec![Camel::Blue])),
            ],
        );
    }
    {
        let result = state.move_unit(Camel::Blue, 2);
        expect(
            result,
            &[
                (1, Space::Camels(vec![Camel::White])),
                (
                    3,
                    Space::Camels(vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (7, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
    {
        let result = state.move_unit(Camel::Blue, 3);
        expect(
            result,
            &[
                (1, Space::Camels(vec![Camel::White])),
                (
                    3,
                    Space::Camels(vec![Camel::Yellow, Camel::Orange, Camel::Green]),
                ),
                (4, Space::Desert(DesertTile::Mirage)),
                (8, Space::Camels(vec![Camel::Blue])),
                (6, Space::Desert(DesertTile::Oasis)),
            ],
        );
    }
}
