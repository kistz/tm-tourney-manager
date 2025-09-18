use base64::Engine;
use dxr::Value;
use serde::Serialize;

use crate::TrackmaniaServer;
use crate::types::XmlRpcMethods;

mod rounds;

#[allow(async_fn_in_trait)]
pub trait ServerConfiguration {
    async fn configure(
        &self, /* options: ExtraConfigOptions *//* , mode: &impl GameModeSettings */
    );
}

pub trait GameModeSettings: Serialize {}

impl ServerConfiguration for TrackmaniaServer {
    async fn configure(&self /* , mode: &impl GameModeSettings */) {
        //let xml_string = quick_xml::se::to_string(mode);

        //println!("Serialized Config: {:?}", xml_string);

        let content = r#"<?xml version="1.0" encoding="utf-8" ?>
<playlist>
	<gameinfos>
		<game_mode>0</game_mode>
		<script_name>Trackmania/TM_Rounds_Online</script_name>
	</gameinfos>

  	<script_settings>
    	<setting name="S_PointsLimit" value="1700" type="integer"/>
    	<setting name="S_RoundsPerMap" value="1" type="integer"/>
    	<setting name="S_MapsPerMatch" value="0" type="integer"/>
    	<setting name="S_UseTieBreak" value="" type="boolean"/>
    	<setting name="S_WarmUpNb" value="0" type="integer"/>
    	<setting name="S_WarmUpDuration" value="60" type="integer"/>
    	<setting name="S_ChatTime" value="10" type="integer"/>
    	<setting name="S_UseClublinks" value="" type="boolean"/>
    	<setting name="S_UseClublinksSponsors" value="" type="boolean"/>
    	<setting name="S_NeutralEmblemUrl" value="" type="text"/>
    	<setting name="S_ScriptEnvironment" value="production" type="text"/>
    	<setting name="S_IsChannelServer" value="" type="boolean"/>
    	<setting name="S_RespawnBehaviour" value="-1" type="integer"/>
    	<setting name="S_HideOpponents" value="" type="boolean"/>
    	<setting name="S_UseLegacyXmlRpcCallbacks" value="1" type="boolean"/>
    	<setting name="S_FinishTimeout" value="15" type="integer"/>
    	<setting name="S_UseAlternateRules" value="" type="boolean"/>
    	<setting name="S_ForceLapsNb" value="-1" type="integer"/>
    	<setting name="S_DisplayTimeDiff" value="" type="boolean"/>
    	<setting name="S_PointsRepartition" value="550, 500, 450, 425, 400, 375, 350, 325, 300, 275, 250, 225, 200, 175, 150, 125, 100, 75, 50, 25, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1" type="text"/>
    	<setting name="S_UseCustomPointsRepartition" value="1" type="boolean"/>
	</script_settings>

	<startindex>0</startindex>
	<map><file>DW25 - Diagram.Map.Gbx</file></map>
    <map><file>DW25 - Acchitchi.Map.Gbx</file></map>
</playlist>"#;
        let loaded = self
            .write_file("MatchSettings/format.txt", content.to_string())
            .await;

        if loaded.is_ok_and(|l| l) {
            _ = self
                .chat_send_server_massage("Tournament mode successfully loaded!")
                .await;

            _ = self.chat_send_server_massage("Starting... GLHF").await;

            _ = self.restart_map().await;
        }

        _ = self.load_match_settings("MatchSettings/format.txt").await;
    }
}
