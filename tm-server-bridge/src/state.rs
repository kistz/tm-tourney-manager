use std::time::Duration;

use spacetimedb_sdk::{Table, Uuid};
use tm_server_controller::{
    callbacks::TypedCallbacks,
    method::{ModeScriptMethodsXmlRpc, XmlRpcMethods},
};
use tm_server_manager_api_rs::{
    RawServerPermittedPlayersTableAccess, RawServerPlayerDestinationTableAccess, post_event,
    post_round_replay, raw_server_player_add,
};
use tm_server_types::{
    base::account_id_to_login,
    event::{EndRoundStart, PlayerConnect, StartMap, StartServer},
};
use tokio::time::sleep;

use crate::{SERVER_METADATA, SPACETIME, TRACKMANIA, TRACKMANIA_FILES};

pub async fn setup_state_synchronization() {
    let server = TRACKMANIA.wait();

    sync_players().await;

    // Sync all events to spacetimedb.
    server.on_event(|event| {
        let spacetime = SPACETIME.read();
        if spacetime
            .reducers
            .post_event(
                //SAFETY: Its the same type. Sadly Rust can not know that :< .
                unsafe {
                    std::mem::transmute::<
                        tm_server_controller::event::Event,
                        tm_server_manager_api_rs::Event,
                    >(event.clone())
                },
            )
            .is_err()
        {
            tracing::error!("Event failed to publish!")
        }
    });

    // Sync the replay of every round to the server.
    server.on_end_round_start(async |event: &EndRoundStart| {
        let file_name = format!("{}{}", event.count, event.time);
        if let Err(error) = server.save_current_replay(&file_name).await {
            tracing::error!("Failed to save Replay File after Round ended. Reason: {error}");
            return;
        };
        let full_path = TRACKMANIA_FILES.wait().clone()
            + "/Replays/"
            + &std::env::var("TM_MASTERSERVER_LOGIN").unwrap()
            + "/Autosaves/"
            + &file_name
            + ".Replay.Gbx";

        match std::fs::read(&full_path) {
            Ok(file) => {
                SPACETIME
                    .read()
                    .procedures
                    .post_round_replay(event.time, file);
            }
            Err(error) => {
                tracing::error!("Failed to read replay file. Reason: {error}")
            }
        };
        if let Err(error) = std::fs::remove_file(&full_path) {
            tracing::error!("Failed to delete the current replay file! Reason: {error}")
        };
    });

    server.on_player_connect(async |event: &PlayerConnect| {
        // Player destination
        move_player_to_destination(Uuid::parse_str(&event.account_id).unwrap()).await;

        // Server allowlist.
        if !SERVER_METADATA.wait().lock().await.open {
            let Some(player) = SPACETIME
                .read()
                .db
                .raw_server_permitted_players()
                .iter()
                .find(|p| Uuid::parse_str(&event.account_id).unwrap() == p.account_id)
            else {
                tracing::warn!("Player tried to connect without the required permissions.");
                if let Err(error) = TRACKMANIA
                    .wait()
                    .kick(event.account_id.clone(), "Not allowed to join the server.")
                    .await
                {
                    tracing::error!("Could not kick player: {error}")
                };

                return;
            };
            if player.only_spectator {
                tracing::warn!(
                    "Player tried to connect as a player but is only allowed as a spectator."
                );
                if let Err(err) = TRACKMANIA
                    .wait()
                    .force_spectator(player.account_id.to_string(), 1)
                    .await
                {
                    tracing::error!("Could not force player to spectator. Error {err}");
                }
            }
        }
    });

    server.on_start_server_start(async |event: &StartServer| {
        let config = unsafe {
            std::mem::transmute::<
                tm_server_manager_api_rs::ServerConfig,
                tm_server_controller::config::ServerConfig,
            >(SERVER_METADATA.wait().lock().await.config.clone())
        };

        let server = TRACKMANIA.wait();

        if event.mode.updated {
            //We need to load the settings again because we changed the script.
            if let Err(error) = server.set_mode_script_settings(config).await {
                tracing::error!("{error}")
            };
            if let Err(err) = server.restart_map().await {
                tracing::error!("Cannot restart!. Reason: {err}");
            };
            tracing::info!("Mode Script was updated");
        } else {
            tracing::info!("Mode Script stayed the same");
            if let Err(err) = server.next_map().await {
                tracing::error!("Cannot go to next map!. Reason: {err}");
            };
        }

        let _: i32 = server
            .call("SaveMatchSettings", format!("{}.txt", event.time))
            .await
            .unwrap();
    });

    /* server.on_begin_match(async |_: &Unit| {
        let config = unsafe {
            std::mem::transmute::<
                tm_server_manager_api_rs::ServerConfig,
                tm_server_controller::config::ServerConfig,
            >(SERVER_METADATA.wait().lock().await.config.clone())
        };

        tokio::spawn(async {
            sleep(Duration::from_secs(2)).await;
            let server = TRACKMANIA.wait();
            if let Err(error) = server.set_mode_script_settings(config).await {
                tracing::error!("{error}")
            };
        });
    }); */

    server.on_start_map_start(async |_: &StartMap| {
        tracing::info!("Reapplying config on StartMapStart.");

        let _: Result<(), tm_server_controller::ClientError> =
            TRACKMANIA.wait().call("GetModeScriptSettings", ()).await;

        let config = unsafe {
            std::mem::transmute::<
                tm_server_manager_api_rs::ServerConfig,
                tm_server_controller::config::ServerConfig,
            >(SERVER_METADATA.wait().lock().await.config.clone())
        };

        if let Err(error) = TRACKMANIA.wait().set_mode_script_settings(config).await {
            tracing::error!("{error}")
        };

        let _: Result<(), tm_server_controller::ClientError> =
            TRACKMANIA.wait().call("GetModeScriptSettings", ()).await;
    });

    server.on_start_map_end(async |_: &StartMap| {
        tracing::info!("Reapplying config on StartMapEnd.");

        let _: Result<(), tm_server_controller::ClientError> =
            TRACKMANIA.wait().call("GetModeScriptSettings", ()).await;

        let config = unsafe {
            std::mem::transmute::<
                tm_server_manager_api_rs::ServerConfig,
                tm_server_controller::config::ServerConfig,
            >(SERVER_METADATA.wait().lock().await.config.clone())
        };

        if let Err(error) = TRACKMANIA.wait().set_mode_script_settings(config).await {
            tracing::error!("{error}")
        };

        let _: Result<(), tm_server_controller::ClientError> =
            TRACKMANIA.wait().call("GetModeScriptSettings", ()).await;
    });
}

/// Synchronizes all the state already present on the server with spacetime db.
pub(super) async fn sync_players() {
    let server = TRACKMANIA.wait();
    let spacetime = SPACETIME.read();
    if let Ok(players) = server.get_player_list().await {
        for player in players {
            // This is the server itself so skip the sync.
            if player.flags & 0b100000 != 0 {
                continue;
            }

            //TODO investigate spectator status return again.
            if player.spectator_status == 0 {
                _ = spacetime
                    .reducers
                    .raw_server_player_add(Uuid::parse_str(&player.account_id).unwrap(), false);
            } else {
                _ = spacetime
                    .reducers
                    .raw_server_player_add(Uuid::parse_str(&player.account_id).unwrap(), true);
            }
        }
    } else {
        tracing::error!(
            "Failed to fetch the player list and thus could not syncronize server state! Aborting.."
        );
        std::process::exit(1)
    }
}

pub fn check_allowed_players() {
    tokio::task::block_in_place(move || {
        tokio::runtime::Handle::current().block_on(async move {
        if !SERVER_METADATA.wait().lock().await.open {
            let server = TRACKMANIA.wait();
            if let Ok(players) = server.get_player_list().await {
                for server_player in players {
                    let Some(player) = SPACETIME
                        .read()
                        .db
                        .raw_server_permitted_players()
                        .iter()
                        .find(|p| {
                            Uuid::parse_str(&server_player.account_id).unwrap() == p.account_id
                        })
                    else {
                        tracing::warn!("Player tried to connect without the required permissions.");
                        if let Err(error) = TRACKMANIA
                            .wait()
                            .kick(
                                server_player.account_id.clone(),
                                "Not allowed to join the server.",
                            )
                            .await
                        {
                            tracing::error!("Could not kick player: {error}")
                        };

                        return;
                    };
                    if player.only_spectator {
                        tracing::warn!(
                            "Player tried to connect as a player but is only allowed as a spectator."
                        );
                        if let Err(err) = TRACKMANIA
                            .wait()
                            .force_spectator(player.account_id.to_string(), 1)
                            .await
                        {
                            tracing::error!("Could not force player to spectator. Error {err}");
                        }
                    }
                }
            }
        }
    });
    });
}

pub fn check_players_have_destination() {
    tokio::task::block_in_place(move || {
        tokio::runtime::Handle::current().block_on(async move {
            let server = TRACKMANIA.wait();
            if let Ok(players) = server.get_player_list().await {
                for server_player in players {
                    move_player_to_destination(Uuid::parse_str(&server_player.account_id).unwrap())
                        .await;
                }
            }
        });
    })
}

pub async fn seamless_recovery() {
    let server = TRACKMANIA.wait();
    let mut seconds = 20;
    while seconds > 0 {
        if let Err(err) = server
            .chat_send_server_massage(format!(
                "[tmservers.live] Match will resume in {seconds} seconds."
            ))
            .await
        {
            tracing::error!("Error sending resume message. Reason: {}", err)
        };
        seconds -= 2;
        sleep(Duration::from_secs(2)).await;
    }

    if let Err(err) = server
        .chat_send_server_massage("[tmservers.live] Match resuming. GLHF.")
        .await
    {
        tracing::error!("Error sending resume message. Reason: {}", err)
    };
    if let Err(err) = server.pause_set_active(false).await {
        tracing::error!(
            "Error unpausing the match because spacetime reconnected. Reson: {}",
            err
        )
    }
}

async fn move_player_to_destination(account_id: Uuid) {
    let server = TRACKMANIA.wait();

    if let Some(player) = SPACETIME
        .read()
        .db
        .raw_server_player_destination()
        .iter()
        .find(|p| account_id == p.account_id)
    {
        let mut seconds = 10;
        while seconds > 0 {
            if let Err(err) = server
                .chat_send_to_account(format!(
                    "[tmservers.live] Found a destination for you you will be automatically moved to a new server in {seconds} seconds."
                ),vec![account_id.to_string()])
                .await
            {
                tracing::error!("Error sending resume message. Reason: {}", err)
            };
            seconds -= 2;
            sleep(Duration::from_secs(2)).await;
        }

        if let Err(error) = server
            .send_open_link_to_account(
                player.account_id.to_string(),
                format!(
                    "#qjoin={}@Trackmania",
                    account_id_to_login(&player.server_account_id.to_string())
                ),
                1,
            )
            .await
        {
            tracing::error!("Could not send link: {error}")
        };
    };
}
