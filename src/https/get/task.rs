use axum::extract::Path;
use axum::{extract::Extension, http::StatusCode, response::IntoResponse, response::Json};
use sqlx::MySqlPool;
use tower_cookies::Cookies;

use crate::models::Task;
use crate::COOKIE_NAME;

pub async fn all_user_tasks_cauth(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
) -> impl IntoResponse {
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());

    let tasks = match sqlx::query_as::<_, Task>(&format!(
        "SELECT * FROM task WHERE user_uuid=\"{}\"",
        user_uuid
    ))
    .fetch_all(&pool)
    .await
    {
        Ok(tasks) => {
            if tasks.is_empty() {
                return (StatusCode::OK, Json(Vec::<i32>::new())).into_response();
            } else {
                tasks
            }
        }
        Err(err) => {
            eprintln!("database error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed while fetching tasks".to_string()),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(tasks)).into_response()
}

pub async fn all_board_tasks_cauth(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
    Path(uuid): Path<String>,
) -> impl IntoResponse {
    let board_uuid = match uuid.is_empty() {
        true => {
            return (
                StatusCode::BAD_REQUEST,
                Json("No board in the request.".to_string()),
            )
                .into_response();
        }
        false => uuid,
    };

    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());

    let tasks = match sqlx::query_as::<_, Task>(&format!(
        "SELECT * FROM task WHERE user_uuid='{}' AND board_uuid = '{}'",
        &user_uuid, &board_uuid
    ))
    .fetch_all(&pool)
    .await
    {
        Ok(tasks) => {
            if tasks.is_empty() {
                return (StatusCode::OK, Json(Vec::<i32>::new())).into_response();
            } else {
                tasks
            }
        }
        Err(err) => {
            eprintln!("database error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed while fetching tasks".to_string()),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(tasks)).into_response()
}
