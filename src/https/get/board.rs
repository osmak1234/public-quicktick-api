use axum::{extract::Extension, http::StatusCode, response::IntoResponse, response::Json};
use sqlx::MySqlPool;
use tower_cookies::Cookies;

use crate::models::Board;
use crate::COOKIE_NAME;

pub async fn all_user_boards_cauth(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
) -> impl IntoResponse {
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());

    let boards = match sqlx::query_as::<_, Board>(&format!(
        "SELECT * FROM board WHERE user_uuid=\"{}\"",
        user_uuid
    ))
    .fetch_all(&pool)
    .await
    {
        Ok(boards) => {
            if boards.is_empty() {
                return (StatusCode::OK, Json(Vec::<i32>::new())).into_response();
            } else {
                boards
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

    (StatusCode::OK, Json(boards)).into_response()
}
