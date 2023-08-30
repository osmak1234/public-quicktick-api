use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::mysql::MySqlPoolOptions;
use tower_cookies::CookieManagerLayer;

use dotenv::dotenv;
use std::{net::SocketAddr, time::Duration};
use tower_http::cors::CorsLayer;

use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, ORIGIN},
    HeaderValue, Method,
};

pub mod helper;
pub mod https;
pub mod models;
pub mod websocket;

pub const COOKIE_NAME: &str = "user_uuid";

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env");

    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers(vec![ORIGIN, AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_origin(vec![
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "https://quicktick-next.vercel.app"
                .parse::<HeaderValue>()
                .unwrap(),
        ]);

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("can't connect to database");
    // build our application with some routes

    let app = Router::new()
        ///////////////
        //task routes//
        ///////////////
        // GET Auth using cookies
        // Return Json{[
        //  Task {
        //      id: i32,
        //      name: String,
        //      description: String,
        //      completed: bool,
        //      user_uuid: String,
        //  }
        //
        // Return Err(message) / OK()
        .route(
            "/get/all_user_tasks",
            get(https::get::task::all_user_tasks_cauth),
        )
        // POST Auth using cookies
        // Input: Json {
        //          name: String,
        //          description: String
        //          uuid: String // (36 chars long)
        //          }
        //
        // Return Ok() / Err(message)
        .route(
            "/post/create_task",
            post(https::post::task::create_task_cauth),
        )
        // DELETE Auth using cookies
        //
        // Input: Using route
        // /delete/task/:id
        // uuid: String
        //
        // Returns Ok() / Err(message)
        .route(
            "/delete/task/:id",
            delete(https::delete::task::delete_task_auth),
        )
        // PATCH Auth using cookies
        //
        // Input Json: {
        //           task_uuid: uuid
        //           action: // One of these strings, and then the optional,
        //                   // 1) RenameTask
        //                   // 2) ChangeDesc
        //                   // 3) ToggleTask
        //                   // 4) ChangeOrder
        //
        //           //1) NewName: String
        //           //2) NewDesc: String
        //           //3)
        //           //4) ChangeOrder: i32
        //         }
        //
        // Returns Ok() / Err(message)
        .route("/patch/task", patch(https::patch::task::handle_task_action))
        .route(
            "/get/all_board_tasks/:uuid",
            get(https::get::task::all_board_tasks_cauth),
        )
        // POST Auth using cookies
        // Input: Json {
        //          name: String,
        //          email: String,
        //          password: String,
        //          }
        //
        // Return Ok() + cookie for auth / Err(message)
        ///////////////
        //user routes//
        ///////////////
        .route("/post/create_user", post(https::post::user::create_user))
        // DELETE Auth using cookies
        //
        // Returns Ok() / Err(message)
        .route("/delete/user", delete(https::delete::user::delete_user))
        // GET
        // destroys cookie auth
        .route("/logout", get(helper::logout))
        // GET login
        // Input: Using route
        // /login/:email/:password
        // email: String
        // password: String
        //
        // Returns Ok() + cookie for auth / Err(message)
        .route("/login/:email/:password", get(helper::login))
        // PATCH Auth using cookies
        //
        // Input Json: {
        //           board_uuid: uuid
        //           action: // One of these strings, and then the optional,
        //                   // 1) RenameBoard
        //
        //           //1) NewName: String
        //         }
        //
        // Returns Ok() / Err(message)
        .route("/patch/user", patch(https::patch::user::handle_user_action))
        ////////////////
        //board routes//
        ////////////////
        // GET all user boards
        //
        // Return Ok(Json{
        //             uuid: String,
        //             name: String,
        //             user_uuid: String,
        //            })
        // / Err(message)
        .route(
            "/get/all_user_board",
            get(https::get::board::all_user_boards_cauth),
        )
        // POST Auth using cookies
        // Input: Json {
        //          name: String,
        //          uuid: String // (36 chars long)
        //          }
        //
        // Return Ok() / Err(message)
        .route("/post/board", post(https::post::board::create_board_cauth))
        // PATCH Auth using cookies
        //
        // Input Json: {
        //           board_uuid: uuid
        //           action: // One of these strings, and then the optional,
        //                   // 1) RenameBoard
        //
        //           //1) NewName: String
        //         }
        //
        // Returns Ok() / Err(message)
        .route(
            "/patch/board",
            patch(https::post::board::create_board_cauth),
        )
        // DELETE Auth using cookies
        //
        // Input: Using route
        // /delete/board/:id
        // uuid: String
        //
        // Returns Ok() / Err(message)
        .route(
            "/delete/board/:id",
            delete(https::delete::board::delete_board_cauth),
        )
        //////////////
        //websockets//
        //////////////
        // GET
        //
        .route("/ws", get(websocket::websocket_handler))
        // Returns HTML with api schema
        .route("/", get(docs))
        // Simple 404 page
        .fallback_service(get(handle_404))
        .layer(Extension(websocket::WebsocketManager::new()))
        .layer(Extension(pool))
        .layer(CookieManagerLayer::new())
        .layer(cors);
    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!(">>> Listening on {addr}\n");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_404() -> impl IntoResponse {
    let html_404 = include_str!("../404.html");
    (StatusCode::NOT_FOUND, Html(html_404))
}

async fn docs() -> impl IntoResponse {
    // Read the contents of the HTML file
    let documentation_html = include_str!("../documentation.html");
    Html(documentation_html)
}
