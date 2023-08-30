use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    response::Json,
};
use sqlx::MySqlPool;
use tower_cookies::Cookies;

use crate::{models::User, COOKIE_NAME};

/// DELETE /
///
pub async fn delete_task_auth(
    Extension(pool): Extension<MySqlPool>,
    //TODO: Extension(websocket_connections): Extension<WebsocketManager>,
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
        "DELETE FROM task WHERE uuid = \"{}\" AND user_uuid = \"{}\"",
        id, user_uuid
    ))
    .execute(&pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                //TODO: Send a websocket message to all clients with uuid "update"
                // websocket_connections.lock().unwrap().send_to(Message::text("update"), "update".to_string());
                (StatusCode::OK, Json(())).into_response()
            } else {
                (StatusCode::NOT_FOUND, Json("Task not found")).into_response()
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Credentials were found, but the deleting of the task errored"),
        )
            .into_response(),
    }
}
