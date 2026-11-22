//! Thin HTTP adapter for the read-only cockpit evaluation.
//!
//! The rules and expected-value calculations live in [`crate::cockpit`].

use crate::cockpit::{evaluate_cockpit, CockpitRequest, CockpitResponse};
use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use serde::Serialize;

const COCKPIT_PATH: &str = "/api/cockpit/actions";

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

pub fn app() -> Router {
    Router::new().route(COCKPIT_PATH, post(evaluate))
}

async fn evaluate(
    Json(request): Json<CockpitRequest>,
) -> Result<Json<CockpitResponse>, impl IntoResponse> {
    match evaluate_cockpit(&request) {
        Ok(response) => Ok(Json(response)),
        Err(message) => Err((StatusCode::BAD_REQUEST, Json(ErrorBody { error: message }))),
    }
}
