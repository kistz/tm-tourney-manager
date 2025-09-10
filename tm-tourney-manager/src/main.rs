use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    response::{Html, IntoResponse},
    routing::get,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use tm_server_client::{
    ClientError, TrackmaniaServer,
    types::{ModeScriptCallbacks, XmlRpcMethods},
};

use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    tx: broadcast::Sender<String>,
    trackmania_server: TrackmaniaServer,
    user_set: Mutex<HashSet<String>>,
}

struct ServerPool {
    servers: Vec<TrackmaniaServer>,
}

struct Event {
    stages: Vec<Stage>,
    next_stage: usize,
    servers: ServerPool,
}

struct Stage {
    starting: Utc,
    status: String,
    participants: Vec<String>,
    matches: Vec<Match>,
}

struct Match {
    server_handle: String,
}

struct Tournament {
    events: Vec<Event>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    //Initialize the Trackmania server
    let server = TrackmaniaServer::new("127.0.0.1:5001").await;

    //let _: Result<bool, ClientError> = server.call("SetApiVersion", "2023-03-24").await;

    let _: Result<bool, ClientError> = server.call("SetApiVersion", "2025-07-04").await;

    let _: Result<bool, ClientError> = server
        .call("Authenticate", ("SuperAdmin", "SuperAdmin"))
        .await;

    let _: Result<bool, ClientError> = server.call("EnableCallbacks", true).await;

    let _: Result<bool, ClientError> = server
        .call(
            "TriggerModeScriptEventArray",
            ("XmlRpc.SetApiVersion", ["3.11"]),
        )
        .await;

    let _: Result<bool, ClientError> = server
        .call(
            "TriggerModeScriptEventArray",
            ("XmlRpc.EnableCallbacks", ["true"]),
        )
        .await;

    let _: Result<bool, ClientError> = server
        .call(
            "TriggerModeScriptEventArray",
            ("XmlRpc.GetMethodsList", ["mhm"]),
        )
        .await;

    println!("{:?}", server.auto_save_replays(true).await);
    println!("{:?}", server.is_auto_save_replays_enabled().await);

    let _: Result<bool, ClientError> = server
        .call("ChatSendServerMessage", "Hey from Rust owo")
        .await;

    server.on_way_point(|info| println!("{info:?}"));

    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState {
        tx,
        trackmania_server: server,
        user_set: Mutex::new(HashSet::new()),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/subscribe/waypoint", get(websocket_handler))
        .route("/admin/{jaaa}", get(index))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Username gets set in the receive loop, if it's valid.
    let mut username = String::new();
    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            // If username that is sent by client is not taken, fill username string.
            check_username(&state, &mut username, name.as_str());

            // If not empty we want to quit the loop else we want to quit function.
            if !username.is_empty() {
                break;
            } else {
                // Only send our client that username is taken.
                let _ = sender
                    .send(Message::Text(Utf8Bytes::from_static(
                        "Username already taken.",
                    )))
                    .await;

                return;
            }
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.tx.subscribe();

    // Now send the "joined" message to all subscribers.
    let msg = format!("{username} joined.");
    tracing::debug!("{msg}");
    let _ = state.tx.send(msg);

    let way_point = state.tx.clone();
    let mut subscription = state
        .trackmania_server
        .subscribe("Trackmania.Event.WayPoint");

    tokio::spawn(async move {
        loop {
            let received = subscription.recv().await.unwrap();
            _ = way_point.send(received.to_string());
        }
    });

    let scores_channel = state.tx.clone();

    state.trackmania_server.on_scores(move |scores| {
        _ = scores_channel.send(serde_json::to_string(&scores).unwrap());
    });

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = state.tx.clone();
    let name = username.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("{name}: {text}"));
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    // Send "user left" message (similar to "joined" above).
    let msg = format!("{username} left.");
    tracing::debug!("{msg}");
    let _ = state.tx.send(msg);

    // Remove username from map so new clients can take it again.
    state.user_set.lock().unwrap().remove(&username);
}

fn check_username(state: &AppState, string: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        string.push_str(name);
    }
}

// Include utf-8 file at **compile** time.
async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
