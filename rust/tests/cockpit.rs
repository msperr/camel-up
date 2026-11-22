use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use camel_up::api::app;
use camel_up::cockpit::{
    evaluate_cockpit, CamelColor, CockpitRequest, CockpitResponse, DesertKind,
};
use camel_up::leg_bets::{expected_leg_bet_coins, AvailableLegBets};
use camel_up::simulation::CAMEL_ORDER;
use camel_up::{Camel, DesertTile, Space, State};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use tower::ServiceExt;

fn request(value: Value) -> CockpitRequest {
    serde_json::from_value(value).expect("valid cockpit request")
}

fn all_tiles(value: u8) -> Value {
    json!({
        "white": value, "yellow": value, "orange": value,
        "green": value, "blue": value,
    })
}

fn test_color(camel: Camel) -> CamelColor {
    match camel {
        Camel::White => CamelColor::White,
        Camel::Yellow => CamelColor::Yellow,
        Camel::Orange => CamelColor::Orange,
        Camel::Green => CamelColor::Green,
        Camel::Blue => CamelColor::Blue,
    }
}

fn odds(response: &CockpitResponse, color: CamelColor) -> &camel_up::CamelOdds {
    response
        .leg_bet
        .camels
        .iter()
        .find(|entry| entry.color == color)
        .expect("every camel is listed")
}

fn colors(response: &CockpitResponse) -> Vec<CamelColor> {
    response
        .leg_bet
        .camels
        .iter()
        .map(|entry| entry.color)
        .collect()
}

#[test]
fn full_board_matches_the_exhaustive_rules_simulation() {
    let snapshot = request(json!({
        "stacks": { "2": ["white"], "5": ["yellow", "orange"], "9": ["green"], "12": ["blue"] },
        "thrownDice": ["white"],
        "availableLegBets": all_tiles(5),
    }));

    let state = State::new(BTreeMap::from([
        (2, Space::Camels(vec![Camel::White])),
        (5, Space::Camels(vec![Camel::Yellow, Camel::Orange])),
        (9, Space::Camels(vec![Camel::Green])),
        (12, Space::Camels(vec![Camel::Blue])),
    ]));
    let (counts, _) = state.simulate_outcomes(&[Camel::White]);
    let total: usize = counts[&Camel::White].iter().sum();
    let available: AvailableLegBets = CAMEL_ORDER.iter().map(|&camel| (camel, vec![5])).collect();
    let expected_ev = expected_leg_bet_coins(&state, &available, &[Camel::White]);

    let result = evaluate_cockpit(&snapshot).expect("valid snapshot");

    for &camel in CAMEL_ORDER.iter() {
        let entry = odds(&result, test_color(camel));
        let p1 = counts[&camel][0] as f64 / total as f64;
        assert!(
            (entry.round_win_probability.unwrap() - p1).abs() < 1e-12,
            "p1 for {:?}",
            camel
        );
        assert!(
            (entry.expected_coins.unwrap() - expected_ev[&camel].unwrap()).abs() < 1e-12,
            "EV for {:?}",
            camel
        );
    }
}

#[test]
fn sorted_by_coins_then_probability_then_colour() {
    let snapshot = request(json!({
        "stacks": {
            "1": ["blue"], "4": ["green"], "8": ["orange"],
            "11": ["yellow"], "15": ["white"]
        },
        "thrownDice": ["white", "yellow", "orange", "green", "blue"],
        "availableLegBets": all_tiles(5),
    }));

    let result = evaluate_cockpit(&snapshot).expect("valid snapshot");

    assert_eq!(
        colors(&result),
        vec![
            CamelColor::White,
            CamelColor::Yellow,
            CamelColor::Orange,
            CamelColor::Green,
            CamelColor::Blue,
        ]
    );
    let evs: Vec<f64> = result
        .leg_bet
        .camels
        .iter()
        .map(|entry| entry.expected_coins.unwrap())
        .collect();
    assert_eq!(evs, vec![5.0, 1.0, -1.0, -1.0, -1.0]);
    let probabilities: Vec<f64> = result
        .leg_bet
        .camels
        .iter()
        .map(|entry| entry.round_win_probability.unwrap())
        .collect();
    assert_eq!(probabilities, vec![1.0, 0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn exhausted_stack_has_no_coins_but_keeps_probability() {
    let snapshot = request(json!({
        "stacks": {
            "1": ["blue"], "4": ["green"], "8": ["orange"],
            "11": ["yellow"], "15": ["white"]
        },
        "thrownDice": ["white", "yellow", "orange", "green", "blue"],
        "availableLegBets": {
            "white": null, "yellow": 5, "orange": 5, "green": 5, "blue": 5
        },
    }));

    let result = evaluate_cockpit(&snapshot).expect("valid snapshot");

    assert_eq!(
        colors(&result),
        vec![
            CamelColor::Yellow,
            CamelColor::Orange,
            CamelColor::Green,
            CamelColor::Blue,
            CamelColor::White,
        ]
    );
    let white = odds(&result, CamelColor::White);
    assert_eq!(white.expected_coins, None);
    assert_eq!(white.round_win_probability, Some(1.0));
}

#[test]
fn dice_ticket_is_worth_one_and_empty_board_is_blank() {
    let snapshot = request(json!({
        "stacks": {},
        "thrownDice": [],
        "availableLegBets": all_tiles(5),
    }));

    let result = evaluate_cockpit(&snapshot).expect("valid snapshot");

    assert_eq!(result.dice.expected_coins, Some(1.0));
    assert!(result
        .leg_bet
        .camels
        .iter()
        .all(|entry| entry.expected_coins.is_none() && entry.round_win_probability.is_none()));
}

#[test]
fn setup_round_suppresses_leg_bet_values() {
    let snapshot = request(json!({
        "stacks": { "2": ["white"], "5": ["yellow", "orange"] },
        "thrownDice": ["white", "yellow", "orange"],
        "availableLegBets": all_tiles(5),
        "setupRound": true,
    }));

    let result = evaluate_cockpit(&snapshot).expect("valid setup snapshot");

    // The setup dice only place the camels: neither action can be taken yet.
    assert_eq!(result.dice.expected_coins, None);
    assert!(!result.leg_bet.available);
    assert_eq!(
        colors(&result),
        vec![
            CamelColor::White,
            CamelColor::Yellow,
            CamelColor::Orange,
            CamelColor::Green,
            CamelColor::Blue,
        ]
    );
    assert!(result
        .leg_bet
        .camels
        .iter()
        .all(|entry| entry.expected_coins.is_none() && entry.round_win_probability.is_none()));
}

#[tokio::test]
async fn http_endpoint_marks_setup_leg_bets_unavailable() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cockpit/actions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "stacks": { "2": ["white"] },
                        "thrownDice": ["white"],
                        "availableLegBets": all_tiles(5),
                        "setupRound": true,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["legBet"]["available"], false);
    assert!(value["dice"]["expectedCoins"].is_null());
    let camels = value["legBet"]["camels"].as_array().unwrap();
    assert_eq!(camels.len(), 5);
    assert!(camels
        .iter()
        .all(|camel| camel["expectedCoins"].is_null() && camel["roundWinProbability"].is_null()));
}

#[test]
fn actions_unavailable_after_five_dice_and_at_race_finish() {
    let five_dice = request(json!({
        "stacks": {
            "1": ["blue"], "4": ["green"], "8": ["orange"],
            "11": ["yellow"], "15": ["white"]
        },
        "thrownDice": ["white", "yellow", "orange", "green", "blue"],
        "availableLegBets": all_tiles(5),
    }));

    let result = evaluate_cockpit(&five_dice).expect("valid snapshot");
    assert!(!result.leg_bet.available);
    assert_eq!(result.dice.expected_coins, None);
    // Final standings are still reported for display.
    assert_eq!(
        odds(&result, CamelColor::White).round_win_probability,
        Some(1.0)
    );

    let finished = request(json!({
        "stacks": { "2": ["white"], "5": ["yellow"], "9": ["green"], "12": ["orange"] },
        "finished": ["blue"],
        "thrownDice": ["white", "yellow", "green", "orange"],
        "availableLegBets": all_tiles(5),
    }));

    let result = evaluate_cockpit(&finished).expect("valid snapshot");
    assert!(!result.leg_bet.available);
    assert_eq!(result.dice.expected_coins, None);
}

#[test]
fn partial_board_gives_stable_probabilities_from_the_first_die() {
    let snapshot = request(json!({
        "stacks": { "2": ["white"] },
        "thrownDice": ["white"],
        "availableLegBets": all_tiles(5),
    }));

    let first = evaluate_cockpit(&snapshot).expect("valid snapshot");
    let second = evaluate_cockpit(&snapshot).expect("valid snapshot");

    assert_eq!(first, second);
    let probability_sum: f64 = first
        .leg_bet
        .camels
        .iter()
        .map(|entry| entry.round_win_probability.unwrap())
        .sum();
    assert!((probability_sum - 1.0).abs() < 1e-12);
    assert!(first
        .leg_bet
        .camels
        .iter()
        .all(|entry| entry.expected_coins.is_some()));
}

#[tokio::test]
async fn http_endpoint_returns_the_evaluation() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cockpit/actions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "stacks": { "2": ["white"] },
                        "thrownDice": ["white"],
                        "availableLegBets": all_tiles(5),
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["dice"]["expectedCoins"], 1.0);
    assert_eq!(value["legBet"]["camels"].as_array().unwrap().len(), 5);
    assert!(value["legBet"]["camels"][0]["color"].is_string());
}

#[tokio::test]
async fn http_endpoint_rejects_duplicate_camels() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cockpit/actions")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "stacks": { "2": ["white"], "3": ["white"] },
                        "thrownDice": [],
                        "availableLegBets": all_tiles(5),
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn rejects_a_die_for_a_camel_that_is_not_on_the_board() {
    let snapshot = request(json!({
        "stacks": { "2": ["white"] },
        "thrownDice": ["blue"],
        "availableLegBets": all_tiles(5),
    }));

    let error = evaluate_cockpit(&snapshot).expect_err("inconsistent snapshot");

    assert!(
        error.contains("not on the board"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_a_partial_board_whose_camels_have_missing_dice() {
    let snapshot = request(json!({
        "stacks": { "2": ["white"] },
        "thrownDice": [],
        "availableLegBets": all_tiles(5),
    }));

    let error = evaluate_cockpit(&snapshot).expect_err("inconsistent snapshot");

    assert!(
        error.contains("needs a thrown die"),
        "unexpected error: {error}"
    );
}

#[test]
fn accepts_an_empty_dice_list_for_a_fully_placed_new_leg() {
    let snapshot = request(json!({
        "stacks": {
            "2": ["white"], "5": ["yellow", "orange"],
            "9": ["green"], "12": ["blue"]
        },
        "thrownDice": [],
        "availableLegBets": all_tiles(5),
    }));

    let state = State::new(BTreeMap::from([
        (2, Space::Camels(vec![Camel::White])),
        (5, Space::Camels(vec![Camel::Yellow, Camel::Orange])),
        (9, Space::Camels(vec![Camel::Green])),
        (12, Space::Camels(vec![Camel::Blue])),
    ]));
    let (counts, _) = state.simulate_outcomes(&[]);
    let total: usize = counts[&Camel::White].iter().sum();
    let result = evaluate_cockpit(&snapshot).expect("a new leg is a valid snapshot");

    for &camel in CAMEL_ORDER.iter() {
        let expected = counts[&camel][0] as f64 / total as f64;
        assert!(
            (odds(&result, test_color(camel))
                .round_win_probability
                .unwrap()
                - expected)
                .abs()
                < 1e-12,
            "p1 for {:?}",
            camel
        );
    }
}

#[test]
fn partial_board_with_an_unreachable_leader_is_deterministic() {
    let snapshot = request(json!({
        "stacks": { "16": ["white"] },
        "thrownDice": ["white"],
        "availableLegBets": all_tiles(5),
    }));

    let result = evaluate_cockpit(&snapshot).expect("valid snapshot");
    let white = odds(&result, CamelColor::White);

    assert_eq!(white.round_win_probability, Some(1.0));
    assert_eq!(white.expected_coins, Some(5.0));
}

#[test]
fn unplaced_camels_do_not_have_a_fixed_stacking_order() {
    let snapshot = request(json!({
        "stacks": { "2": ["white"] },
        "thrownDice": ["white"],
        "availableLegBets": all_tiles(5),
    }));

    let result = evaluate_cockpit(&snapshot).expect("valid snapshot");
    let probabilities: Vec<f64> = [
        CamelColor::Yellow,
        CamelColor::Orange,
        CamelColor::Green,
        CamelColor::Blue,
    ]
    .iter()
    .map(|&color| odds(&result, color).round_win_probability.unwrap())
    .collect();

    let max = probabilities.iter().copied().fold(0.0_f64, f64::max);
    let min = probabilities.iter().copied().fold(1.0_f64, f64::min);
    let spread = max - min;
    assert!(
        spread < 0.03,
        "exchangeable unplaced camels should have similar win probabilities, got {probabilities:?}"
    );
}

async fn post(body: serde_json::Value) -> StatusCode {
    app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cockpit/actions")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn http_endpoint_rejects_an_invalid_leg_betting_tile() {
    let status = post(json!({
        "stacks": { "2": ["white"] },
        "thrownDice": ["white"],
        "availableLegBets": { "white": 7 },
    }))
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn http_endpoint_rejects_an_invalid_space() {
    let status = post(json!({
        "stacks": { "0": ["white"] },
        "thrownDice": [],
        "availableLegBets": {},
    }))
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn http_endpoint_rejects_an_inconsistent_dice_list() {
    let status = post(json!({
        "stacks": { "2": ["white"] },
        "thrownDice": ["blue"],
        "availableLegBets": all_tiles(5),
    }))
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Only blue's die is still in the pyramid: from space 9 its three possible
/// rolls land on 10, 11 and 12, giving exact 1/3 incomes there.
fn pending_blue_snapshot() -> Value {
    json!({
        "stacks": {
            "2": ["yellow"], "3": ["orange"], "5": ["white"],
            "7": ["green"], "9": ["blue"]
        },
        "thrownDice": ["white", "yellow", "orange", "green"],
        "availableLegBets": all_tiles(5),
    })
}

fn pending_blue_state() -> State {
    State::new(BTreeMap::from([
        (2, Space::Camels(vec![Camel::Yellow])),
        (3, Space::Camels(vec![Camel::Orange])),
        (5, Space::Camels(vec![Camel::White])),
        (7, Space::Camels(vec![Camel::Green])),
        (9, Space::Camels(vec![Camel::Blue])),
    ]))
}

#[test]
fn deserts_deserialise_and_default_to_empty() {
    let mut with_tiles = pending_blue_snapshot();
    with_tiles["deserts"] = json!([
        { "space": 8, "kind": "oasis" },
        { "space": 11, "kind": "mirage" }
    ]);
    let snapshot = request(with_tiles);

    assert_eq!(snapshot.deserts.len(), 2);
    assert_eq!(snapshot.deserts[0].space, 8);
    assert_eq!(snapshot.deserts[0].kind, DesertKind::Oasis);
    assert_eq!(snapshot.deserts[1].space, 11);
    assert_eq!(snapshot.deserts[1].kind, DesertKind::Mirage);
    assert!(request(pending_blue_snapshot()).deserts.is_empty());
}

#[test]
fn desert_options_have_exact_coins_and_expected_ordering() {
    let result = evaluate_cockpit(&request(pending_blue_snapshot())).expect("valid snapshot");
    let options = &result.desert.options;

    // Ten legal spaces x two faces; the three reachable spaces come first.
    assert_eq!(options.len(), 20);
    let top: Vec<(u8, DesertKind, f64)> = options[..6]
        .iter()
        .map(|option| (option.space, option.kind, option.expected_coins))
        .collect();
    assert_eq!(
        top,
        vec![
            (10, DesertKind::Oasis, 1.0 / 3.0),
            (10, DesertKind::Mirage, 1.0 / 3.0),
            (11, DesertKind::Oasis, 1.0 / 3.0),
            (11, DesertKind::Mirage, 1.0 / 3.0),
            (12, DesertKind::Oasis, 1.0 / 3.0),
            (12, DesertKind::Mirage, 1.0 / 3.0),
        ]
    );
    assert!(options[6..]
        .iter()
        .all(|option| option.expected_coins == 0.0));

    let spaces: Vec<u8> = options[6..].iter().map(|option| option.space).collect();
    assert_eq!(
        spaces,
        vec![4, 4, 6, 6, 8, 8, 13, 13, 14, 14, 15, 15, 16, 16]
    );
}

#[test]
fn desert_income_matches_exhaustive_simulation_with_existing_tiles() {
    let mut snapshot = pending_blue_snapshot();
    snapshot["thrownDice"] = json!(["white", "yellow", "orange"]);
    snapshot["deserts"] = json!([{ "space": 11, "kind": "oasis" }]);

    let mut data = pending_blue_state().data;
    data.insert(11, Space::Desert(DesertTile::Oasis));
    let mut with_candidate = data.clone();
    with_candidate.insert(8, Space::Desert(DesertTile::Oasis));

    let (counts, hits) =
        State::new(with_candidate).simulate_outcomes(&[Camel::White, Camel::Yellow, Camel::Orange]);
    let total: usize = counts[&Camel::White].iter().sum();
    let expected = hits.get(&8).copied().unwrap_or(0) as f64 / total as f64;

    let result = evaluate_cockpit(&request(snapshot)).expect("valid snapshot");
    let option = result
        .desert
        .options
        .iter()
        .find(|option| option.space == 8 && option.kind == DesertKind::Oasis)
        .expect("space 8 is legal");
    assert!(
        (option.expected_coins - expected).abs() < 1e-12,
        "got {}, expected {}",
        option.expected_coins,
        expected
    );
}

#[test]
fn desert_options_skip_occupied_and_adjacent_spaces() {
    let mut snapshot = pending_blue_snapshot();
    snapshot["deserts"] = json!([{ "space": 8, "kind": "mirage" }]);

    let result = evaluate_cockpit(&request(snapshot)).expect("valid snapshot");
    let spaces: BTreeMap<u8, usize> =
        result
            .desert
            .options
            .iter()
            .fold(BTreeMap::new(), |mut acc, option| {
                *acc.entry(option.space).or_insert(0) += 1;
                acc
            });

    // Tile on 8 forbids 7, 8 and 9; the camels occupy 2, 3, 5, 7 and 9.
    assert_eq!(
        spaces.keys().copied().collect::<Vec<_>>(),
        vec![4, 6, 10, 11, 12, 13, 14, 15, 16]
    );
    assert!(spaces.values().all(|&count| count == 2));
}

#[test]
fn desert_options_are_empty_at_leg_end_and_after_the_finish() {
    let mut leg_end = pending_blue_snapshot();
    leg_end["thrownDice"] = json!(["white", "yellow", "orange", "green", "blue"]);
    assert!(evaluate_cockpit(&request(leg_end))
        .expect("valid snapshot")
        .desert
        .options
        .is_empty());

    let finished = request(json!({
        "stacks": {
            "2": ["yellow"], "3": ["orange"], "5": ["white"], "7": ["green"]
        },
        "finished": ["blue"],
        "thrownDice": ["white", "yellow", "orange", "green"],
        "availableLegBets": all_tiles(5),
    }));
    assert!(evaluate_cockpit(&finished)
        .expect("valid snapshot")
        .desert
        .options
        .is_empty());

    let empty = request(json!({
        "stacks": {},
        "thrownDice": [],
        "availableLegBets": all_tiles(5),
    }));
    assert!(evaluate_cockpit(&empty)
        .expect("valid snapshot")
        .desert
        .options
        .is_empty());
}

#[test]
fn setup_round_has_no_desert_options() {
    let mut setup = pending_blue_snapshot();
    setup["setupRound"] = json!(true);

    let result = evaluate_cockpit(&request(setup)).expect("valid setup snapshot");
    assert!(result.desert.options.is_empty());
    assert_eq!(result.dice.expected_coins, None);
    assert!(!result.leg_bet.available);
}

#[test]
fn partial_board_desert_options_are_deterministic() {
    let snapshot = request(json!({
        "stacks": { "2": ["white"] },
        "thrownDice": ["white"],
        "availableLegBets": all_tiles(5),
    }));

    let first = evaluate_cockpit(&snapshot).expect("valid snapshot");
    let second = evaluate_cockpit(&snapshot).expect("valid snapshot");

    assert_eq!(first.desert.options, second.desert.options);
    assert!(!first.desert.options.is_empty());
}

#[tokio::test]
async fn http_endpoint_rejects_invalid_deserts() {
    let invalid = [
        json!({ "space": 1, "kind": "oasis" }),
        json!({ "space": 0, "kind": "oasis" }),
        json!({ "space": 17, "kind": "oasis" }),
        json!({ "space": 5, "kind": "oasis" }), // camel
    ];
    for desert in invalid {
        let mut snapshot = pending_blue_snapshot();
        snapshot["deserts"] = json!([desert]);
        assert_eq!(post(snapshot).await, StatusCode::BAD_REQUEST, "{desert}");
    }

    let mut duplicate = pending_blue_snapshot();
    duplicate["deserts"] = json!([
        { "space": 11, "kind": "oasis" },
        { "space": 11, "kind": "mirage" }
    ]);
    assert_eq!(post(duplicate).await, StatusCode::BAD_REQUEST);

    let mut adjacent = pending_blue_snapshot();
    adjacent["deserts"] = json!([
        { "space": 11, "kind": "oasis" },
        { "space": 12, "kind": "mirage" }
    ]);
    assert_eq!(post(adjacent).await, StatusCode::BAD_REQUEST);

    // Unknown JSON enum values are rejected by serde before the handler runs.
    let mut bad_kind = pending_blue_snapshot();
    bad_kind["deserts"] = json!([{ "space": 10, "kind": "volcano" }]);
    assert_eq!(post(bad_kind).await, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn http_endpoint_returns_sorted_desert_options_and_accepts_legacy_requests() {
    let mut snapshot = pending_blue_snapshot();
    snapshot["deserts"] = json!([{ "space": 8, "kind": "oasis" }]);
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cockpit/actions")
                .header("content-type", "application/json")
                .body(Body::from(snapshot.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    let options = value["desert"]["options"]
        .as_array()
        .expect("options array");
    assert_eq!(options[0]["space"], 10);
    assert_eq!(options[0]["kind"], "oasis");
    assert!((options[0]["expectedCoins"].as_f64().unwrap() - 1.0 / 3.0).abs() < 1e-12);
    assert!(!options.iter().any(|option| option["space"] == 8));

    let legacy = post(pending_blue_snapshot()).await;
    assert_eq!(legacy, StatusCode::OK);
}
