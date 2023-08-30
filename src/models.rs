use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct Task {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub completed: bool,
    pub user_uuid: String,
    pub board_uuid: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Board {
    pub uuid: String,
    pub name: String,
    pub user_uuid: String,
    pub special: Option<i32>,
}
