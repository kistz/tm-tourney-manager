use nadeo_api::{NadeoRequest, auth::AuthType, request::Method};
use serde::{Deserialize, Serialize};
use tm_server_client::{
    ClientError,
    types::{XmlRpcMethods, config::MapPoolConfig},
};
use tm_tourney_manager_api_rs::TmServer;
use tracing::info;

use crate::{NADEO, TRACKMANIA};

pub async fn configure(server_state: TmServer) {
    let local_server = TRACKMANIA.wait();

    //SAFETY: Same type but rust cant know that.
    let configuration = unsafe {
        std::mem::transmute::<
            tm_tourney_manager_api_rs::ServerConfig,
            tm_server_client::types::config::ServerConfig,
        >(server_state.config)
    };

    let config = configuration.into_xml();

    let written = local_server
        .write_file("MatchSettings/manager.txt", config.into())
        .await;

    //Configuration was successfully saved.
    if written.is_ok_and(|r| r) {
        // Load all maps to make them accessible locally
        get_maps(configuration.iter_maps()).await;
    }

    let loaded = local_server
        .load_match_settings("MatchSettings/manager.txt")
        .await;

    //TODO figure out what the returned integer means.
    //if loaded.is_ok_and(|l| l == 2) {
    if loaded.is_ok() {
        _ = local_server
            .chat_send_server_massage("[tm-server-bridge]   Server state synchronized.")
            .await;

        _ = local_server.next_map().await;
    }
}

pub(crate) async fn get_maps(maps: impl Iterator<Item = &String>) {
    //Only used in this function
    #[derive(Debug, Serialize, Deserialize)]
    struct MapInfo {
        #[serde(rename = "fileUrl")]
        file_url: String,
        #[serde(rename = "mapUid")]
        map_uid: String,
        name: String,
    }

    for map in maps {
        let req = NadeoRequest::builder()
            .method(Method::GET)
            .auth_type(AuthType::NadeoServices)
            .url(&format!(
                "https://prod.trackmania.core.nadeo.online/maps/?mapUidList={map}"
            ))
            .build()
            .unwrap();
        let resp = NADEO.wait().lock().await.execute(req).await;

        let map_info: Vec<MapInfo> = resp.unwrap().json().await.unwrap();
        let map_info = &map_info[0];

        let req = NadeoRequest::builder()
            .method(Method::GET)
            .auth_type(AuthType::NadeoServices)
            .url(&map_info.file_url)
            .build()
            .unwrap();

        let resp = NADEO.wait().lock().await.execute(req).await;
        let map_file = resp.unwrap().bytes().await.unwrap();
        _ = TRACKMANIA
            .wait()
            .write_file(&format!("{}.Map.Gbx", &map_info.map_uid), map_file.to_vec())
            .await;
        let _: Result<bool, ClientError> = TRACKMANIA
            .wait()
            .call(
                "ChatSendServerMessage",
                format!("[tm-server-bridge]   Imported map: {}$fff.", &map_info.name),
            )
            .await;
    }
}
