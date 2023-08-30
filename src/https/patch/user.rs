use axum::{
    extract::{self, Extension},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use tower_cookies::Cookies;

use crate::COOKIE_NAME;

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    ChangeName(String),
}

pub async fn handle_user_action(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
    extract::Json(request): extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());

    let action = match request.get("action").and_then(|v| v.as_str()) {
        Some("NewName") => {
            let new_name = request.get("new_name").and_then(|v| v.as_str());
            match new_name {
                Some(val) => Action::ChangeName(val.to_string()),
                None => {
                    return (StatusCode::BAD_REQUEST, Json("Invalid JSON body")).into_response();
                }
            }
        }
        _ => return (StatusCode::BAD_REQUEST, Json("Invalid JSON body")).into_response(),
    };

    let query = match action {
        Action::ChangeName(new_name) => format!(
            "UPDATE user SET name = \"{}\" WHERE uuid = \"{}\"",
            &new_name, &user_uuid
        ),
    };

    match sqlx::query(&query).execute(&pool).await {
        Ok(_) => (StatusCode::OK, Json(())).into_response(),
        Err(err) => {
            eprintln!("{}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed while processing task action"),
            )
                .into_response()
        }
    }
}
