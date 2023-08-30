use std::sync::{Arc, Mutex};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use http::StatusCode;
use tower_cookies::Cookies;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    cookies: Cookies,
    Extension(websockets): Extension<WebsocketManager>,
) -> impl IntoResponse {
    let uuid = cookies.get("user_uuid").unwrap().value().to_string();

    // I need to upgrade the websocket connection and then add it to the list of connections so I can send messages from other routes.

    (StatusCode::OK, ()).into_response()
}

pub struct WebsocketManager {
    connection: Arc<Mutex<Vec<(String, WebSocket)>>>,
}

impl Clone for WebsocketManager {
    fn clone(&self) -> Self {
        WebsocketManager {
            connection: self.connection.clone(),
        }
    }
}

impl Default for WebsocketManager {
    fn default() -> Self {
        WebsocketManager::new()
    }
}

impl WebsocketManager {
    pub fn new() -> Self {
        WebsocketManager {
            connection: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_connection(&self, uuid: String, ws: WebSocket) -> usize {
        let mut connections = self.connection.lock().unwrap();
        let index = connections.len();
        connections.push((uuid, ws));
        index
    }

    pub fn remove_connection(&self, index: usize) {
        self.connection.lock().unwrap().remove(index);
    }

    pub fn send_to(&self, message: Message, uuid: String) {
        let _ = self
            .connection
            .lock()
            .unwrap()
            .iter_mut()
            .map(|(ws_uuid, ws)| async {
                if *ws_uuid == uuid {
                    let _ = ws.send(message.clone()).await;
                }
            });
    }
}
