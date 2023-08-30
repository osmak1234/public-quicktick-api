use axum::{extract, extract::Extension, http::StatusCode, response::IntoResponse, response::Json};
use bcrypt::hash;
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{MySqlPool, Row};
use tower_cookies::{cookie::SameSite, Cookie, Cookies};

extern crate bcrypt;

use crate::models::User;

/// POST /post/create_user
/// Input: Json {
///   email: string
///   password: string
///   name: string
/// }
///
/// Output:
pub async fn create_user(
    Extension(pool): Extension<MySqlPool>,
    cookies: Cookies,
    extract::Json(request): extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let (email, password, name) = match (
        request.get("email").and_then(|v| v.as_str()),
        request.get("password").and_then(|v| v.as_str()),
        request.get("name").and_then(|v| v.as_str()),
    ) {
        (Some(email), Some(password), Some(name)) => {
            (email.to_string(), password.to_string(), name.to_string())
        }
        _ => {
            eprintln!("Invalid JSON body: {:?}", request);
            return (
                StatusCode::BAD_REQUEST,
                Json("Incorrect json body. Check the docs for correct version."),
            )
                .into_response();
        }
    };

    // Check if the email already exists if err return early, else continue
    match sqlx::query_as::<_, User>(&format!("SELECT * FROM user WHERE email =\"{}\"", email))
        .fetch_optional(&pool)
        .await
    {
        Ok(val) => {
            if val.is_some() {
                //invalid creation email already exists
                return (
                    StatusCode::BAD_REQUEST,
                    Json("Email already exists".to_string()),
                )
                    .into_response();
            }
        }
        Err(err) => {
            eprintln!("database error: {:?}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Account with that email already exists".to_string()),
            )
                .into_response();
        }
    };

    let salt: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let salted_password = format!("{}{}", password, salt);
    let hashed_password = hash(salted_password, 10).unwrap();
    // First, perform the INSERT operation
    let insert_query = format!(
        "INSERT INTO user (email, password, name, salt) VALUES ('{}', '{}', '{}', '{}')",
        &email, &hashed_password, &name, &salt
    );

    if let Err(err) = sqlx::query(&insert_query).execute(&pool).await {
        eprintln!("Insert error: {}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Account creation failed"),
        )
            .into_response();
    }


    // Next, perform the SELECT operation
    let select_query = format!(
        "SELECT uuid FROM user WHERE email = '{}' AND password = '{}'",
        &email, &hashed_password
    );

    let user_uuid: String = match sqlx::query(&select_query).fetch_one(&pool).await {
        Ok(row) => row.get("uuid"),
        Err(err) => {
            eprintln!("Select error: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Account doesn't exist, but creation failed"),
            )
                .into_response();
        }
    };

    match sqlx::query(&format!(
        "INSERT INTO board (name, user_uuid, special ) VALUES ( '{}' , '{}', '{}' ), ( '{}' , '{}', '{}' )",
        "Home", user_uuid, 1,
        "Archive", user_uuid, 2
    ))
    .execute(&pool)
    .await
    {
        Ok(_) => {},
        Err(err) => {
            eprintln!("{}", err);
                return 
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Failed while creating task".to_string()),
            )
                .into_response();
        }
    };

    let cookie = Cookie::build("user_uuid", user_uuid)
        .path("/")
        .domain("quicktick-api.fly.dev")
        .same_site(SameSite::None)
        .finish();
    cookies.add(cookie);

    (StatusCode::OK, Json(())).into_response()
}
