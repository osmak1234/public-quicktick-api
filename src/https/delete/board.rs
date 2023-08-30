use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    response::Json,
};
use sqlx::MySqlPool;
use tower_cookies::Cookies;

use crate::{models::User, COOKIE_NAME};

pub async fn delete_board_cauth(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());
    match sqlx::query_as::<_, User>(&format!("SELECT * FROM user WHERE uuid =\"{}\"", user_uuid))
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json("There aren't any matching credentials. Try loggin in again.".to_string()),
            )
                .into_response();
        }
        Err(err) => {
            eprintln!("database error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed while looking for credentials".to_string()),
            )
                .into_response();
        }
    }

    match sqlx::query(&format!(
        "DELETE board, task FROM board
         LEFT JOIN task ON board.uuid = task.board_uuid
         WHERE (board.uuid = '{}' OR task.board_uuid = '{}') AND board.user_uuid = '{}'",
        &id, &id, &user_uuid
    ))
    .execute(&pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                (StatusCode::OK, Json(())).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json("Board not found")).into_response()
            }
        }
        Err(err) => {
            eprintln!("Database err: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Authorized , but the deleting of the board failed"),
            )
                .into_response()
        }
    }
}
