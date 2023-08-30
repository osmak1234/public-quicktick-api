use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    response::Json,
};
use sqlx::MySqlPool;

use crate::{models::User, COOKIE_NAME};
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

extern crate bcrypt;
use bcrypt::verify;

pub async fn login(
    Path((email, password)): Path<(String, String)>,
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
) -> impl IntoResponse {
    let user_uuid = cookies
        .get(COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or("FAILED".to_string());

    if user_uuid != *"FAILED" && email == *"cookie" && password == *"cookie" {
        match sqlx::query_as::<_, User>(&format!(
            "SELECT * FROM user WHERE uuid = \"{}\"",
            user_uuid
        ))
        .fetch_one(&pool)
        .await
        {
            Ok(res) => (StatusCode::OK, Json(res)).into_response(),
            Err(err) => {
                eprintln!("database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Failed while using cookie as login. Relogin please.".to_string()),
                )
                    .into_response()
            }
        }
    } else {
        let user_uuid = match sqlx::query_as::<_, User>(&format!(
            "SELECT * FROM user WHERE email =\"{}\" ",
            email
        ))
        .fetch_optional(&pool)
        .await
        {
            Ok(Some(user)) => {
                let salted_password = format!("{}{}", password, user.salt);
                let valid = verify(salted_password, &user.password);
                if valid.is_ok() {
                    user.uuid
                } else {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json("There isn't a matching password, email was found".to_string()),
                    )
                        .into_response();
                }
            }
            Ok(None) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json("There isn't any matching email".to_string()),
                )
                    .into_response();
            }
            Err(err) => {
                eprintln!("database error: {:?}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Failed while looking for email".to_string()),
                )
                    .into_response();
            }
        };
        // create new cookie
        let cookie = Cookie::build("user_uuid", user_uuid)
            .path("/")
            .domain("quicktick-api.fly.dev")
            .same_site(SameSite::None)
            .finish();
        cookies.add(cookie);

        (StatusCode::OK, Json("Logged in".to_string())).into_response()
    }
}

pub async fn logout(cookies: Cookies) -> impl IntoResponse {
    let cookie = Cookie::build("user_uuid", "")
        .path("/")
        .domain("quicktick-api.fly.dev")
        .same_site(SameSite::None)
        .finish();

    cookies.remove(cookie);
    (StatusCode::OK, Json(())).into_response()
}
