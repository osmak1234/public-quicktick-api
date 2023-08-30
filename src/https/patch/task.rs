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
    RenameTask(String),
    ChangeDesc(String),
    ToggleTask,
    ChangeOrder(i32),
    MoveBoard(String),
}

pub async fn handle_task_action(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
    extract::Json(request): extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());

    let (task_uuid, enum_action) = match (
        request.get("action").and_then(|v| v.as_str()),
        request.get("task_uuid").and_then(|v| v.as_str()),
    ) {
        (Some("RenameTask"), Some(task_uuid)) => {
            let new_name = request
                .get("NewName")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            (
                task_uuid.to_string(),
                Action::RenameTask(new_name.to_string()),
            )
        }
        (Some("ChangeDesc"), Some(task_uuid)) => {
            let new_desc = request
                .get("NewDesc")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            (
                task_uuid.to_string(),
                Action::ChangeDesc(new_desc.to_string()),
            )
        }
        (Some("ToggleTask"), Some(task_uuid)) => (task_uuid.to_string(), Action::ToggleTask),
        (Some("ChangeOrder"), Some(task_uuid)) => {
            let new_order = request
                .get("NewOrder")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            (task_uuid.to_string(), Action::ChangeOrder(new_order))
        }
        (Some("MoveBoard"), Some(task_uuid)) => {
            let new_board = request
                .get("NewBoard")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            (
                task_uuid.to_string(),
                Action::MoveBoard(new_board.to_string()),
            )
        }

        _ => return (StatusCode::BAD_REQUEST, Json("Invalid JSON body")).into_response(),
    };

    let query = match &enum_action {
        Action::RenameTask(new_name) => format!(
            "UPDATE task SET name = \"{}\" WHERE uuid = \"{}\" AND user_uuid = \"{}\"",
            new_name, task_uuid, user_uuid
        ),
        Action::ChangeDesc(new_desc) => format!(
            "UPDATE task SET description = \"{}\" WHERE uuid = \"{}\" AND user_uuid = \"{}\"",
            new_desc, task_uuid, user_uuid
        ),
        Action::ToggleTask => format!(
            "UPDATE task SET completed = NOT completed WHERE uuid = \"{}\" AND user_uuid = \"{}\"",
            task_uuid, user_uuid
        ),
        Action::ChangeOrder(new_order) => format!(
            "UPDATE task SET order = \"{}\" WHERE uuid = \"{}\" AND user_uuid = \"{}\"",
            new_order, task_uuid, user_uuid
        ),
        Action::MoveBoard(new_board) => format!(
            "UPDATE task SET board_uuid = '{}' WHERE uuid = '{}' AND user_uuid = '{}'",
            new_board, task_uuid, user_uuid
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
