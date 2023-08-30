use axum::{extract::Extension, http::StatusCode, response::IntoResponse, response::Json};
use sqlx::MySqlPool;
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

use crate::COOKIE_NAME;

pub async fn delete_user(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
) -> impl IntoResponse {
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());
    println!("{}", user_uuid);

    match (
        sqlx::query(&format!(
            "DELETE FROM `task` WHERE user_uuid = \"{}\"",
            &user_uuid
        ))
        .execute(&pool)
        .await,
        sqlx::query(&format!(
            "DELETE FROM `user` WHERE uuid = \"{}\" ",
            &user_uuid
        ))
        .execute(&pool)
        .await,
    ) {
        (Ok(_), Ok(_)) => {
            let cookie = Cookie::build("user_uuid", "")
                .path("/")
                .domain("quicktick-api.fly.dev")
                .same_site(SameSite::None)
                .finish();

            cookies.remove(cookie);
            (StatusCode::OK, Json(())).into_response()
        }
        (Ok(_), Err(err)) => {
            eprintln!("Error deleting user from user table: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error deleting user from user table".to_string()),
            )
                .into_response()
        }
        (Err(err), Ok(_)) => {
            cookies.remove(Cookie::new(COOKIE_NAME, ""));
            eprintln!("Error deleting user from task table: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error deleting user from task table".to_string()),
            )
                .into_response()
        }
        (Err(err1), Err(err2)) => {
            eprintln!(
                "Error deleting user from both tables: User Table - {:?}, Task Table - {:?}",
                err1, err2
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Error deleting user from both tables".to_string()),
            )
                .into_response()
        }
    }
}
