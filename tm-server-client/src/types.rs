use std::{
    cell::OnceCell,
    sync::{Arc, LazyLock, Mutex, OnceLock},
};

use base64::{Engine, engine::general_purpose::STANDARD};

use serde::{Deserialize, Serialize};
//use tm_server_types::WayPointEvent;

use crate::{ClientError, TrackmaniaServer};

pub use tm_server_types::*;

/* #[derive(Debug, Serialize, Deserialize)]
pub struct WayPointEvent {
    #[serde(rename = "accountid")]
    account_id: String,
    login: String,
    time: u32,
    racetime: u32,
    laptime: u32,
    speed: f32,

    #[serde(rename = "checkpointinrace")]
    checkpoint_in_race: u32,
    #[serde(rename = "checkpointinlap")]
    checkpoint_in_lap: u32,
    #[serde(rename = "isendrace")]
    is_end_race: bool,
    #[serde(rename = "isendlap")]
    is_end_lap: bool,
    #[serde(rename = "isinfinitelaps")]
    is_infinite_laps: bool,
    #[serde(rename = "isindependentlaps")]
    is_independent_laps: bool,
    #[serde(rename = "curracecheckpoints")]
    current_race_checkpoints: Vec<u32>,
    #[serde(rename = "curlapcheckpoints")]
    current_lap_checkpoints: Vec<u32>,
    #[serde(rename = "blockid")]
    block_id: String,
} */

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoresEvent {
    #[serde(rename = "responseid")]
    response_id: String,
    section: String,
    #[serde(rename = "useteams")]
    use_teams: bool,

    #[serde(rename = "winnerteam")]
    winner_team: i32,
    #[serde(rename = "winnerplayer")]
    winner_player: String,

    teams: Vec<Team>,
    players: Vec<Player>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    login: String,
    #[serde(rename = "accountid")]
    account_id: String,

    name: String,

    team: i32,

    rank: u32,

    #[serde(rename = "roundpoints")]
    round_points: u32,
    #[serde(rename = "mappoints")]
    map_points: u32,
    #[serde(rename = "matchpoints")]
    match_points: u32,

    #[serde(rename = "bestracetime")]
    best_racetime: u32,

    #[serde(rename = "bestracecheckpoints")]
    best_race_checkpoints: Vec<u32>,
    #[serde(rename = "bestlaptime")]
    best_laptime: u32,
    #[serde(rename = "bestlapcheckpoints")]
    best_lap_checkpoints: Vec<u32>,
    #[serde(rename = "prevracetime")]
    previous_racetime: u32,
    #[serde(rename = "prevracecheckpoints")]
    previous_race_checkpoints: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Team {
    id: u32,
    name: String,
    #[serde(rename = "roundpoints")]
    round_points: u32,
    #[serde(rename = "mappoints")]
    map_points: u32,
    #[serde(rename = "matchpoints")]
    match_points: u32,
}

pub trait ModeScriptCallbacks {
    fn on_way_point(&self, execute: impl Fn(&WayPointEvent) + Send + Sync + 'static);
    fn on_scores(&self, execute: impl Fn(&ScoresEvent) + Send + Sync + 'static);
}

impl ModeScriptCallbacks for TrackmaniaServer {
    fn on_way_point(&self, execute: impl Fn(&WayPointEvent) + Send + Sync + 'static) {
        self.on("Trackmania.Event.WayPoint", execute);
    }

    fn on_scores(&self, execute: impl Fn(&ScoresEvent) + Send + Sync + 'static) {
        self.on("Trackmania.Scores", execute);
    }
}

#[allow(async_fn_in_trait)]
pub trait ModeScriptMethodsXmlRpc {
    async fn enable_callbacks(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_callbacks_list(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_callbacks_list_enabled(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_callbacks_list_disabled(&self, enable: bool) -> Result<bool, ClientError>;
    async fn block_callbacks(&self, enable: bool) -> Result<bool, ClientError>;
    async fn unblock_callbacks(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_callback_help(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_methods_list(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_method_help(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_doscumentation(&self, enable: bool) -> Result<bool, ClientError>;
    async fn set_api_version(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_api_version(&self, enable: bool) -> Result<bool, ClientError>;
    async fn get_all_api_versions(&self, enable: bool) -> Result<bool, ClientError>;
}

impl ModeScriptMethodsXmlRpc for TrackmaniaServer {
    ///Enable or disable mode script callbacks.
    async fn enable_callbacks(&self, enable: bool) -> Result<bool, ClientError> {
        self.call(
            "TriggerModeScriptEventArray",
            ("XmlRpc.EnableCallbacks", ["true"]),
        )
        .await
    }

    async fn get_callbacks_list(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_callbacks_list_enabled(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_callbacks_list_disabled(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn block_callbacks(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn unblock_callbacks(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_callback_help(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_methods_list(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_method_help(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_doscumentation(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn set_api_version(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_api_version(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }

    async fn get_all_api_versions(&self, enable: bool) -> Result<bool, ClientError> {
        todo!()
    }
}

#[allow(async_fn_in_trait)]
pub trait XmlRpcMethods {
    async fn kick(&self, player: String, message: Option<String>) -> Result<bool, ClientError>;

    async fn add_guest(&self, player: &str) -> Result<bool, ClientError>;

    async fn auto_save_replays(&self, enable: bool) -> Result<bool, ClientError>;

    async fn is_auto_save_replays_enabled(&self) -> Result<bool, ClientError>;

    async fn save_current_replay(&self, path: &str) -> Result<bool, ClientError>;

    async fn write_file(&self, path: &str, content: String) -> Result<bool, ClientError>;

    async fn load_match_settings(&self, path: &str) -> Result<i32, ClientError>;

    async fn chat_send_server_massage(&self, message: &str) -> Result<bool, ClientError>;

    async fn restart_map(&self) -> Result<bool, ClientError>;
}

impl XmlRpcMethods for TrackmaniaServer {
    async fn kick(&self, player: String, message: Option<String>) -> Result<bool, ClientError> {
        todo!()
    }

    async fn add_guest(&self, login: &str) -> Result<bool, ClientError> {
        self.call("AddGuest", login).await
    }

    async fn auto_save_replays(&self, enable: bool) -> Result<bool, ClientError> {
        self.call("AutoSaveReplays", enable).await
    }

    async fn is_auto_save_replays_enabled(&self) -> Result<bool, ClientError> {
        self.call("IsAutoSaveReplaysEnabled", ()).await
    }

    async fn save_current_replay(&self, path: &str) -> Result<bool, ClientError> {
        self.call("IsAutoSaveReplaysEnabled", path).await
    }

    async fn write_file(&self, path: &str, content: String) -> Result<bool, ClientError> {
        self.call("WriteFile", (path, content.into_bytes())).await
    }

    async fn load_match_settings(&self, path: &str) -> Result<i32, ClientError> {
        self.call("LoadMatchSettings", path).await
    }

    async fn chat_send_server_massage(&self, message: &str) -> Result<bool, ClientError> {
        self.call("ChatSendServerMessage", message).await
    }

    async fn restart_map(&self) -> Result<bool, ClientError> {
        self.call("RestartMap", ()).await
    }
}
