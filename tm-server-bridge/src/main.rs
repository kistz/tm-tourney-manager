use std::{
    collections::VecDeque,
    sync::{LazyLock, Mutex as StdMutex, OnceLock},
};

use nadeo_api::NadeoClient;

use spacetimedb_sdk::{DbContext, Error, EventTable, Table, Uuid};

use tm_server_controller::{
    TrackmaniaServer,
    method::{ModeScriptMethodsXmlRpc, XmlRpcMethods},
};
use tm_server_manager_api_rs::{
    DbConnection, ErrorContext, EventRawServerMethodTableAccess, EventRawServerState,
    EventRawServerStateTableAccess, RawServerPermittedPlayersTableAccess,
    RawServerPlayerDestinationTableAccess, event_raw_server_methodQueryTableAccess,
    event_raw_server_stateQueryTableAccess, login_as_server,
    raw_server_permitted_playersQueryTableAccess, raw_server_player_destinationQueryTableAccess,
};
use tm_server_types::event::Event;
use tokio::{signal, sync::Mutex};
use tracing::instrument;

use crate::{
    chat::setup_chat,
    config::metadata_update,
    methods::method_call_received,
    state::{
        check_allowed_players, check_players_have_destination, setup_state_synchronization,
        spacetime_disconnected,
    },
};

mod chat;
mod config;
mod methods;
mod state;

#[cfg(test)]
mod test;

/// Exposes the associated trackmania server globally.
static TRACKMANIA: OnceLock<TrackmaniaServer> = OnceLock::new();

/// Exposes the SpacetimeDB connection.
static SPACETIME: OnceLock<DbConnection> = OnceLock::new();

/// Exposes the NadeoAPI with server auth.
static NADEO: OnceLock<Mutex<NadeoClient>> = OnceLock::new();

/// Path to the Filesystem of the trackmnia server UserData.
static TRACKMANIA_FILES: OnceLock<String> = OnceLock::new();

static SERVER_METADATA: OnceLock<Mutex<EventRawServerState>> = OnceLock::new();
static EVENT_CACHE: LazyLock<StdMutex<VecDeque<Event>>> =
    LazyLock::new(|| StdMutex::new(VecDeque::with_capacity(1000)));

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt};

pub(crate) fn init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("DEBUG_LOG_LEVEL")
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Err(dbg_error) = dotenvy::from_path(env!("CARGO_MANIFEST_DIR").to_string() + "/.env")
        && let Err(prod_error) = dotenvy::dotenv()
    {
        tracing::warn!("No .env file was found. Error Prod: {prod_error}, Error Dbg: {dbg_error}")
    };

    init_tracing_subscriber();

    let tm_server_login = std::env::var("TM_MASTERSERVER_LOGIN")
        .expect("Environment variable: TM_MASTERSERVER_LOGIN MUST be set");
    let tm_server_password = std::env::var("TM_MASTERSERVER_PASSWORD")
        .expect("Environment variable: TM_MASTERSERVER_password MUST be set");
    std::env::var("TM_ACCOUNT_ID")
        .expect("Environment variable: TM_ACCOUNT_ID MUST be set at the moment.
        This will be the account where the server will be available under and can be obtained from e.g. trackmania.io. 
        We hope to make this optional in the future but depend on a change from nadeo on that sooo good luck ^^");
    let tm_server_url = std::env::var("TM_SERVER_URL").expect("Environment variable: TM_SERVER_URL MUST be set. This is needed to connect to the Trackmania server.");

    TRACKMANIA_FILES
        .set(std::env::var("TM_FILES").unwrap_or("./UserData".into()))
        .expect("The Path to the Trackmania Filesystem could not be established. Aborting.");

    if !std::fs::exists(TRACKMANIA_FILES.wait()).is_ok_and(|b| b) {
        panic!(
            "The TM_FILES variable is set to {} but the directory does not exist.
            Consider mounting the correct directory if you are using a docker container.",
            TRACKMANIA_FILES.wait()
        );
    } else {
        tracing::info!("Successfully detected the trackmania filesystem.");
    }

    {
        //Initialize the NadeoClient
        let nadeo = NadeoClient::builder()
            .with_server_auth(&tm_server_login, &tm_server_password)
            .user_agent("tm-server-bridge")
            .build()
            .await
            .unwrap();
        _ = NADEO.set(nadeo.into());
    }

    //Connect to the Trackmania server
    {
        if let Ok(server) = TrackmaniaServer::new(tm_server_url.clone()).await {
            _ = TRACKMANIA.set(server);
        } else {
            let server = TrackmaniaServer::new(tm_server_url).await?;
            _ = TRACKMANIA.set(server);
        }
    }

    spacetime_connect(false).await;

    // Initial Configuration for the Trackmania server connection.
    {
        let server = TRACKMANIA.wait();

        let _: bool = server.call("SetApiVersion", "2023-03-25").await?;

        server.authenticate("SuperAdmin", "SuperAdmin").await?;

        let _: bool = server
            .call(
                "TriggerModeScriptEventArray",
                ("XmlRpc.SetApiVersion", ["3.11"]),
            )
            .await?;

        server.enable_callbacks(true).await?;
        server.enable_mode_script_callbacks(true).await?;

        server.chat_manual_routing(true, false).await?;
    }

    setup_state_synchronization().await;
    setup_chat().await;

    // Initialize state subscriptions for the server.
    {}

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            tracing::error!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }

    Ok(())
}

/* fn on_stdb_connect_error(_ctx: &ErrorContext, err: Error) {
    tracing::error!("SpacetimeDB connection error: {:?}", err);
    std::process::exit(1);
} */

fn on_stdb_disconnected(_: &ErrorContext, err: Option<Error>) {
    if let Some(err) = err {
        tracing::error!(
            "Forcefully disconnected from SpacetimeDB with Error: {}",
            err
        );
        //let connection = DbConnection::custom_new(ctx.imp());
        return;
    }
    tracing::error!("Disconnected from spacetimedb.");
    spacetime_disconnected();
}

#[instrument(level = "debug")]
async fn spacetime_connect(seamless: bool) -> bool {
    let Ok(spacetime) = DbConnection::builder()
        //.on_connect_error(on_stdb_connect_error)
        .on_disconnect(on_stdb_disconnected)
        .with_database_name(std::env::var("SPACETIMEDB_MODULE").unwrap_or("tmservers".to_string()))
        .with_uri(
            std::env::var("SPACETIMEDB_URL")
                .unwrap_or("wss://maincloud.spacetimedb.com".to_string()),
        )
        .build()
    else {
        tracing::error!("Server could not be connected successfully");
        return false;
    };

    let tm_server_login = std::env::var("TM_MASTERSERVER_LOGIN").unwrap();
    let tm_server_password = std::env::var("TM_MASTERSERVER_PASSWORD").unwrap();
    let tm_account_id = std::env::var("TM_ACCOUNT_ID").unwrap();
    let tm_account_id = Uuid::parse_str(&tm_account_id).unwrap();

    _ = SPACETIME.set(spacetime);

    tokio::spawn(async move {
        let connection = SPACETIME.wait();
        loop {
            _ = connection.run_async().await;
        }
    });

    let Ok(server_id) = SPACETIME
        .wait()
        .procedures()
        .login_as_server_async(tm_server_login, tm_server_password, tm_account_id, seamless)
        .await
        .unwrap()
    else {
        tracing::error!("Server could not be authenticated successfully");
        return false;
    };

    tracing::info!("Successfully connected to tmservers.live!");

    // Initialize state subscriptions for the server.

    let spacetime = SPACETIME.wait();

    _ = spacetime
        .subscription_builder()
        .on_applied(|_| tracing::debug!("Subscription successfully applied!"))
        .on_error(|_, error| tracing::error!("Subscription failed: {error:?}"))
        .add_query(|ctx| {
            ctx.from
                .event_raw_server_method()
                .r#where(|s| s.server_id.eq(server_id))
        })
        .add_query(|ctx| {
            ctx.from
                .event_raw_server_state()
                .r#where(|s| s.server_id.eq(server_id))
        })
        .add_query(|ctx| ctx.from.raw_server_permitted_players())
        .add_query(|ctx| ctx.from.raw_server_player_destination())
        .subscribe();

    spacetime
        .db
        .event_raw_server_state()
        .on_insert(metadata_update);

    spacetime
        .db
        .raw_server_permitted_players()
        .on_delete(|_, _| check_allowed_players());

    spacetime
        .db
        .event_raw_server_method()
        .on_insert(method_call_received);

    spacetime
        .db
        .raw_server_player_destination()
        .on_insert(|_, _| check_players_have_destination());

    true
}
