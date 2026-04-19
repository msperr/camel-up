use camel_cup::{Camel, DesertTile, Field, State};
use std::collections::BTreeMap;
use std::panic::catch_unwind;

fn mk_map(entries: &[(i32, Field)]) -> BTreeMap<i32, Field> {
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
        let res = catch_unwind(|| state.move_camel(Camel::White, steps));
        assert!(res.is_err(), "expected panic for case {}", case_desc);
    };

    // 1) pos3 = Oasis: test pos4 desert variants (both should panic), then empty and orange
    {
        // base initial map: white at pos1 and oasis at pos3
        let base = vec![
            (1, Field::Camels(vec![Camel::White])),
            (3, Field::Desert(DesertTile::Oasis)),
        ];

        // deserts at pos4 -> both Oasis and Mirage should panic (precondition)
        for tile in &[DesertTile::Oasis, DesertTile::Mirage] {
            let mut entries = base.clone();
            entries.push((4, Field::Desert(*tile)));
            let state = State::new(mk_map(&entries));
            expect_panic(state, &format!("pos3=Oasis pos4={:?}", tile));
        }

        // pos4 empty -> white is forwarded to 4, desert hit is Some(3)
        {
            let state = State::new(mk_map(&base));
            let (res_state, desert_hit) = state.move_camel(Camel::White, steps);
            assert_eq!(desert_hit, Some(3));
            let expected = vec![
                (3, Field::Desert(DesertTile::Oasis)),
                (4, Field::Camels(vec![Camel::White])),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }

        // pos4 has Orange -> white forwarded to 4 and appended -> [Orange, White]
        {
            let mut entries = base.clone();
            entries.push((4, Field::Camels(vec![Camel::Orange])));
            let state = State::new(mk_map(&entries));
            let (res_state, desert_hit) = state.move_camel(Camel::White, steps);
            assert_eq!(desert_hit, Some(3));
            let expected = vec![
                (3, Field::Desert(DesertTile::Oasis)),
                (4, Field::Camels(vec![Camel::Orange, Camel::White])),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }
    }

    // 2) pos3 = Mirage: test pos2 desert variants (both should panic), then empty and orange
    {
        let base = vec![
            (1, Field::Camels(vec![Camel::White])),
            (3, Field::Desert(DesertTile::Mirage)),
        ];

        // deserts at pos2 -> both Oasis and Mirage should panic (precondition)
        for tile in &[DesertTile::Oasis, DesertTile::Mirage] {
            let mut entries = base.clone();
            entries.push((2, Field::Desert(*tile)));
            let state = State::new(mk_map(&entries));
            expect_panic(state, &format!("pos3=Mirage pos2={:?}", tile));
        }

        // pos2 empty -> white moved back to 2, desert hit is Some(3)
        {
            let state = State::new(mk_map(&base));
            let (res_state, desert_hit) = state.move_camel(Camel::White, steps);
            assert_eq!(desert_hit, Some(3));
            let expected = vec![
                (2, Field::Camels(vec![Camel::White])),
                (3, Field::Desert(DesertTile::Mirage)),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }

        // pos2 has Orange -> white prepended to 2 -> [White, Orange]
        {
            let mut entries = base.clone();
            entries.push((2, Field::Camels(vec![Camel::Orange])));
            let state = State::new(mk_map(&entries));
            let (res_state, desert_hit) = state.move_camel(Camel::White, steps);
            assert_eq!(desert_hit, Some(3));
            let expected = vec![
                (2, Field::Camels(vec![Camel::White, Camel::Orange])),
                (3, Field::Desert(DesertTile::Mirage)),
            ];
            assert_eq!(res_state.data, mk_map(&expected));
        }
    }

    // 3) pos3 = empty: white just lands on pos3 normally
    {
        let base = vec![(1, Field::Camels(vec![Camel::White]))];
        let state = State::new(mk_map(&base));
        let (res_state, desert_hit) = state.move_camel(Camel::White, steps);
        assert_eq!(desert_hit, None);
        let expected = vec![(3, Field::Camels(vec![Camel::White]))];
        assert_eq!(res_state.data, mk_map(&expected));
    }

    // 4) pos3 = Yellow: white lands on pos3 appended -> [Yellow, White]
    {
        let base = vec![
            (1, Field::Camels(vec![Camel::White])),
            (3, Field::Camels(vec![Camel::Yellow])),
        ];
        let state = State::new(mk_map(&base));
        let (res_state, desert_hit) = state.move_camel(Camel::White, steps);
        assert_eq!(desert_hit, None);
        let expected = vec![(3, Field::Camels(vec![Camel::Yellow, Camel::White]))];
        assert_eq!(res_state.data, mk_map(&expected));
    }
}
