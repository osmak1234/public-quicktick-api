use axum::{extract, extract::Extension, http::StatusCode, response::IntoResponse, response::Json};
use sqlx::MySqlPool;

use tower_cookies::Cookies;

use crate::{models::User, COOKIE_NAME};

pub async fn create_board_cauth(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
    extract::Json(request): extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let (name, uuid) = match (
        request.get("name").and_then(|v| v.as_str()),
        request.get("uuid").and_then(|v| v.as_str()),
    ) {
        (Some(name), Some(uuid)) => (name.to_string(), uuid.to_string()),
        _ => {
            eprintln!("Invalid JSON body: {:?}", request);
            return (
                StatusCode::BAD_REQUEST,
                Json("Incorrect json body. Check the docs for correct version."),
            )
                .into_response();
        }
    };

    // verify user_uuid
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());

    match sqlx::query_as::<_, User>(&format!(
        "SELECT * FROM user WHERE uuid = \"{}\" ",
        &user_uuid
    ))
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json("There isn't any matching account, please login again.".to_string()),
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
    };

    match sqlx::query_as::<_, User>(&format!("SELECT * FROM board WHERE uuid = \"{}\" ", &uuid))
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(_)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Something went wrong, try again.".to_string()),
            )
                .into_response();
        }
        Ok(None) => {}
        Err(err) => {
            eprintln!("database error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed while creating a board.".to_string()),
            )
                .into_response();
        }
    };

    match sqlx::query(&format!(
        "INSERT INTO board (name, user_uuid, uuid ) VALUES ( '{}' , '{}', '{}' )",
        name, user_uuid, uuid,
    ))
    .execute(&pool)
    .await
    {
        Ok(_) => (StatusCode::OK, Json(())).into_response(),
        Err(err) => {
            eprintln!("{}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed while creating task".to_string()),
            )
                .into_response()
        }
    }
}
