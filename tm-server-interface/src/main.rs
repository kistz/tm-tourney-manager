use std::sync::OnceLock;

use nadeo_api::NadeoClient;
use spacetimedb_sdk::{
    DbContext, Error, Event as StdbEvent, Identity, Status, Table, TableWithPrimaryKey,
};

use tm_tourney_manager_api::*;

use tm_server_client::{ClientError, TrackmaniaServer, configurator::ServerConfiguration};
use tokio::signal;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// The URI of the SpacetimeDB instance hosting our chat database and module.
const HOST: &str = "http://localhost:1234";

/// The database name we chose when we published our module.
const DB_NAME: &str = "tourney-manager";

const TM_SERVER_ID: &str = "test";

/// Exposes the associated trackmania server globally.
static SERVER: OnceLock<TrackmaniaServer> = OnceLock::new();
static SPACETIME: OnceLock<DbConnection> = OnceLock::new();

/// Load credentials from a file and connect to the database.
fn connect_to_db() -> DbConnection {
    /* let nando_auth = NadeoClient::builder()
    .with_server_auth("joestestcellar", r#"O#2nvOW^6+Y,\*CS"#)
    .build()
    .await
    .unwrap(); */
    //let TOKEN = "eyJhbGciOiJIUzUxMiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICIwMDAwZWQ0MS1iODc1LTQ5NGMtYmMxOS02MTc4YWVjMWFhNzYifQ.eyJleHAiOjE3NTkwMDM1MTcsImlhdCI6MTc1ODk2NzUxNywianRpIjoiNDg3MWYwYzctODkzZi1jYTE5LTNmZWItM2NmMWQyM2ExZTliIiwiaXNzIjoiaHR0cDovL2xvY2FsaG9zdDo1Njc4L3JlYWxtcy9tYXN0ZXIiLCJzdWIiOiI2MjRkMWFmNC1iMTY2LTQ1MmYtYjdjYi0wZGM2YmY5NzZlNTAiLCJ0eXAiOiJTZXJpYWxpemVkLUlEIiwic2lkIjoiZWNhYjdkNTUtOTMwMC00YjlhLTkxNmQtMjE2ZWQxNjRmMWM3Iiwic3RhdGVfY2hlY2tlciI6Ik5WTC1KRWFKb0Z2YlF6eVpSdHJQc18xSHY0WGp6Vk1qbldxQUF1ZXF1OG8ifQ.kgOwfQqPfYKDZMZTn0VYhAGl8Jm68TZGcCDErvNKYZni6cBEP3Cy6Ukly7uxq_omzrVOhBFoher1szDFZ6aL_A";

    DbConnection::builder()
        // Register our `on_connect` callback, which will save our auth token.
        .on_connect(on_connected)
        // Register our `on_connect_error` callback, which will print a message, then exit the process.
        .on_connect_error(on_connect_error)
        // Our `on_disconnect` callback, which will print a message, then exit the process.
        .on_disconnect(on_disconnected)
        // If the user has previously connected, we'll have saved a token in the `on_connect` callback.
        // In that case, we'll load it and pass it to `with_token`,
        // so we can re-authenticate as the same `Identity`.
        //.with_token(Some(TOKEN))
        // Set the database name we chose when we called `spacetime publish`.
        .with_module_name(DB_NAME)
        // Set the URI of the SpacetimeDB host that's running our database.
        .with_uri(HOST)
        // Finalize configuration and connect!
        .build()
        .expect("Failed to connect")
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

    {
        //Initialize the Trackmania server
        let server = TrackmaniaServer::new("127.0.0.1:5001").await;
        _ = SERVER.set(server);
    }

    let server = SERVER.wait();

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
            "ChatSendServerMessage",
            "Server Interface connected successfully :>",
        )
        .await;

    // Connect to the database
    {
        let spacetime = connect_to_db();
        _ = SPACETIME.set(spacetime);
    }

    let spacetime = SPACETIME.wait();

    _ = spacetime
        .subscription_builder()
        .on_applied(|_| tracing::debug!("Subscription successfully applied!"))
        .on_error(|_, mhm| tracing::error!("Subscription failed: {mhm:?}"))
        .subscribe(format!(
            "SELECT * FROM tm_server WHERE id = '{TM_SERVER_ID}'"
        ));

    _ = spacetime.reducers.add_server(TM_SERVER_ID.into());

    spacetime.db.tm_server().on_update(server_update);

    server.configure().await;

    server.event(move |event| {
        let spacetime = SPACETIME.wait();
        if spacetime
            .reducers
            .post_event(
                TM_SERVER_ID.into(),
                //SAFETY: Its the same type. Sadly Rust can not know that :< .
                unsafe {
                    std::mem::transmute::<
                        tm_server_client::types::event::Event,
                        tm_tourney_manager_api::Event,
                    >(event.clone())
                },
            )
            .is_err()
        {
            println!("Event failed to publish!")
        }
    });

    // Spawn a thread, where the connection will process messages and invoke callbacks.
    tokio::spawn(async move {
        loop {
            _ = spacetime.run_async().await;
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }

    Ok(())
}

/// Our `on_connect` callback: save our credentials to a file.
fn on_connected(_ctx: &DbConnection, _identity: Identity, token: &str) {
    /* if let Err(e) = creds_store().save(token) {
        eprintln!("Failed to save credentials: {:?}", e);
    } */
    println!("Token connected: {token}");
}

/// Our `on_connect_error` callback: print the error, then exit the process.
fn on_connect_error(_ctx: &ErrorContext, err: Error) {
    eprintln!("Connection error: {:?}", err);
    std::process::exit(1);
}

/// Our `on_disconnect` callback: print a note, then exit the process.
fn on_disconnected(_ctx: &ErrorContext, err: Option<Error>) {
    if let Some(err) = err {
        eprintln!("Disconnected: {}", err);
        std::process::exit(1);
    } else {
        println!("Disconnected.");
        std::process::exit(0);
    }
}

// Register all the callbacks our app will use to respond to database events.
/* fn register_callbacks(ctx: &DbConnection) {
    // When a new user joins, print a notification.
    ctx.db.user().on_insert(on_user_inserted);

    // When a user's status changes, print a notification.
    ctx.db.user().on_update(on_user_updated);

    // When a new message is received, print it.
    ctx.db.message().on_insert(on_message_inserted);

    // When we fail to set our name, print a warning.
    ctx.reducers.on_set_name(on_name_set);

    // When we fail to send a message, print a warning.
    ctx.reducers.on_send_message(on_message_sent);
} */

fn server_update(_: &EventContext, _: &TmServer, new: &TmServer) {
    let server = SERVER.wait();
    let paused = new.server_command.pause;

    tokio::spawn(async move {
        let _: Result<bool, ClientError> =
            server.call("ChatSendServerMessage", "Method called").await;

        let _: Result<bool, ClientError> = server
            .call(
                "TriggerModeScriptEventArray",
                (
                    "Maniaplanet.Pause.SetActive",
                    [if paused { "true" } else { "false" }],
                ),
            )
            .await;
    });
}
