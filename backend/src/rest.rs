use crate::pow::PowValidator;
use crate::search::ItemRepo;
use crate::{services, Repositories};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spow::pow::Pow;
use std::collections::HashMap;

pub async fn search_post(
    State(repo): State<Repositories>,
    Query(search_params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let search_page = search_params
        .get("page")
        .unwrap_or(&"1".to_owned())
        .parse::<usize>()
        .unwrap();
    let search_query = search_params
        .get("search")
        .unwrap_or(&"".to_owned())
        .to_string();
    let result = services::find_posts(repo.db, search_query.as_str(), search_page).await;
    if result.is_err() {
        tracing::error!("{:?}", result.unwrap_err());
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Json(result.unwrap()).into_response()
}

pub async fn get_post(
    State(repo): State<Repositories>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let result = services::find_post(repo.db.get_db(), slug).await;
    if result.is_err() {
        tracing::error!("{:?}", result.unwrap_err().reason);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Json(result.unwrap()).into_response()
}

pub async fn get_challenge_form() -> impl IntoResponse {
    // Create a new PoW which will be valid for 60 seconds
    let pows: Vec<_> = (0..16)
        .map(|_i| Pow::with_difficulty(18, 600).unwrap().to_string())
        .collect();

    // Create a puzzle challenge from this pow.
    // You can either call `build_challenge()` or `.to_string()`.
    Json(json!({"challenges": pows})).into_response()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmitForm {
    body: String,
    challenges: [String; 16],
}

pub async fn post_form(
    State(repo): State<Repositories>,
    Json(submit): Json<SubmitForm>,
) -> impl IntoResponse {
    if !repo
        .db
        .get_pow_validator()
        .is_valid_pow(submit.challenges)
        .await
    {
        return StatusCode::BAD_REQUEST;
    }
    tracing::info!("{:?}", submit.body);
    StatusCode::OK
}
