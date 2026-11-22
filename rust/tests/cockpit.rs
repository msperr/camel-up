use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use camel_up::api::app;
use camel_up::cockpit::{evaluate_cockpit, CamelColor, CockpitRequest, CockpitResponse};
use camel_up::leg_bets::{expected_leg_bet_coins, AvailableLegBets};
use camel_up::simulation::CAMEL_ORDER;
use camel_up::{Camel, Space, State};
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

    assert_eq!(result.dice.expected_coins, 1.0);
    assert!(result
        .leg_bet
        .camels
        .iter()
        .all(|entry| entry.expected_coins.is_none() && entry.round_win_probability.is_none()));
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
