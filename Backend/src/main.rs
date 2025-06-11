use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::{extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::{
    net::TcpListener, sync::{broadcast, mpsc, Mutex}, time::interval
};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    connections: Arc<Mutex<HashMap<String, broadcast::Sender<Message>>>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        connections: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state);

    let listner = TcpListener::bind("0.0.0.0:8000")
        .await
        .expect("Failed to bind to address");
    println!("Server running on ws://0.0.0.0:8000");

    axum::serve(listner, app)
        .await
        .expect("Failed to start server");
} 

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let conn_id = Uuid::new_v4().to_string();
    let conn_id_clone = conn_id.clone();

    let (tx, mut rx) = broadcast::channel(100);
    {
        let mut connections = state.connections.lock().await;
        connections.insert(conn_id, tx.clone());
    }

    let (mut sender, mut receiver) = socket.split();
    let (message_tx, mut message_rx) = mpsc::channel::<Message>(100);

    let sender_task = tokio::spawn(async move {
        while let Some(msg) = message_rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let ping_tx = message_tx.clone();
    let ping_task = tokio::spawn(async move {
        let mut intervel = interval(Duration::from_secs(30));
        loop {
            intervel.tick().await;
            if ping_tx.send(Message::Ping(vec![])).await.is_err() {
                break;
            }
        }
    });

    let forward_tx = message_tx.clone();
    let forward_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if forward_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    
    let receive_task = tokio::spawn({
        let tx_clone = tx.clone();
        let conn_id_for_task = conn_id_clone.clone();
        let state = state.clone();
        let mut target_map: HashMap<String, String> = HashMap::new();

        async move {
            while let Some(msg_result) = receiver.next().await {
                match msg_result {
                    Ok(msg) => match msg {
                        Message::Text(text) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                if data["type"] == "register" {
                                    if let Some(id) = data["connectionId"].as_str() {
                                        state
                                            .connections
                                            .lock()
                                            .await
                                            .insert(id.to_string(), tx_clone.clone());
                                    }
                                    continue;
                                }

                                if let Some(target_id) = data["target_id"].as_str() {
                                    target_map
                                        .insert(conn_id_for_task.clone(), target_id.to_string());
                                    if let Some(target_tx) =
                                        state.connections.lock().await.get(target_id)
                                    {
                                        let _ = target_tx.send(Message::Text(text));
                                    }
                                }
                            }
                        }
                        Message::Binary(bin_data) => {
                            if let Some(target_id) = target_map.get(&conn_id_for_task) {
                                if let Some(target_tx) =
                                    state.connections.lock().await.get(target_id)
                                {
                                    let _ = target_tx.send(Message::Binary(bin_data));
                                } else {
                                    println!("Target connection not found: {}", target_id);
                                }
                            }
                        }
                        Message::Close(_) => break,
                        _ => continue,
                    },
                    Err(e) => {
                        eprintln!("WebSocket error: {:?}", e);
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = sender_task => {},
        _ = ping_task => {},
        _ = forward_task => {},
        _ = receive_task => {},
    }

    state.connections.lock().await.remove(&conn_id_clone);
    println!("Connection {} closed\n", conn_id_clone);
}
