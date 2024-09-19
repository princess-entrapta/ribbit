use crate::pow::PowValidator;
use crate::schemas::PostEntity;
use crate::search::ItemRepo;
use crate::services::register_post;
use crate::{services, Repositories};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{Form, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spow::pow::Pow;
use std::collections::HashMap;

pub async fn search_post(
    State(repo): State<Repositories>,
    Path(lang): Path<String>,
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
    match repo.hb.render("list", &result.unwrap()) {
        Ok(html) => Html::from(html).into_response(),
        Err(err) => {
            tracing::error!("{:?}", err.to_string());
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_post(
    State(repo): State<Repositories>,
    Path((lang, slug)): Path<(String, String)>,
) -> impl IntoResponse {
    let result = services::find_post(repo.db.get_db(), slug).await;
    if result.is_err() {
        tracing::error!("{:?}", result.unwrap_err().reason);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    match repo.hb.render("post", &result.unwrap()) {
        Ok(html) => Html::from(html).into_response(),
        Err(err) => {
            tracing::error!("{:?}", err.to_string());
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn home(State(repo): State<Repositories>, Path(lang): Path<String>) -> impl IntoResponse {
    match repo.hb.render("home", &json!({})) {
        Ok(html) => Html::from(html).into_response(),
        Err(err) => {
            tracing::error!("{:?}", err.to_string());
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn get_challenge_form(State(repo): State<Repositories>) -> impl IntoResponse {
    // Create a new PoW which will be valid for 15 minutes
    let pows: Vec<_> = (0..16)
        .map(|_i| Pow::with_difficulty(18, 900).unwrap().to_string())
        .collect();

    match repo.hb.render("publish", &json!({"challenges": pows})) {
        Ok(html) => Html::from(html).into_response(),
        Err(err) => {
            tracing::error!("{:?}", err.to_string());
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublishForm {
    pub body: String,
    pub title: String,
    pub visibility_group: Option<uuid::Uuid>,
    pub reply_group: Option<uuid::Uuid>,
    pub tags: String,
    pub challenges: [String; 16],
}

pub async fn post_form(
    State(repo): State<Repositories>,
    Json(submit): Json<PublishForm>,
) -> impl IntoResponse {
    if !repo
        .db
        .get_pow_validator()
        .is_valid_pow(submit.challenges.clone())
        .await
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    tracing::info!("{:?}", submit.body.clone());
    let post = PostEntity::from_form(submit);
    match register_post(repo.db, post.clone()).await {
        Ok(_) => post.slug.into_response(),
        Err(err) => {
            tracing::error!("{:?}", err.to_string());
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
