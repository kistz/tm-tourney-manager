mod rounds;
use std::collections::BTreeMap;

pub use rounds::Rounds;

mod reverse_cup;
pub use reverse_cup::ReverseCup;

mod reverse_cup_v2;
pub use reverse_cup_v2::ReverseCupV2;

mod time_attack;
pub use time_attack::TimeAttack;

mod knockout;
pub use knockout::Knockout;

mod rounds_bot_online;
pub use rounds_bot_online::RoundsBotOnline;

mod common;
pub use common::*;

mod options;
pub use options::ServerOptions;

mod helper;
pub use helper::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct ServerConfig {
    // Dedicated Config TODO
    options: ServerOptions,

    // Playlist settings.
    common: Common,
    mode: ModeSettings,
    maps: MapPoolConfig,
}

impl ServerConfig {
    pub fn into_xml(&self) -> String {
        r#"<?xml version="1.0" encoding="utf-8" ?>
<playlist>
	<gameinfos>
		<game_mode>0</game_mode>
		"#
        .to_string()
            + &self.mode.mode_header()
            + r#"
    </gameinfos>

  	<script_settings>"#
            + &self.common.into_xml()
            + &self.mode.into_xml()
            + r#"
	</script_settings>
	"# + &self.maps.into_xml()
            + "
</playlist>"
    }

    pub fn get_common(&self) -> &Common {
        &self.common
    }

    pub fn get_mode(&self) -> &ModeSettings {
        &self.mode
    }

    pub fn get_maps(&self) -> &MapPoolConfig {
        &self.maps
    }

    pub fn script_name(&self) -> &str {
        self.mode.script_name()
    }

    pub fn iter_maps(&self) -> impl Iterator<Item = &String> {
        self.maps.map_uids.iter()
    }

    pub fn get_mode_settings_struct(&self) -> dxr::Value {
        let mut cfg = self.common.get_xml_map();
        cfg.append(&mut self.mode.get_xml_map());
        dxr::Value::Struct(cfg)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            common: Common::default_rounds(),
            mode: ModeSettings::Rounds(Rounds::default()),
            maps: Default::default(),
            options: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub enum ModeSettings {
    Rounds(Rounds),
    ReverseCup(ReverseCup),
    TimeAttack(TimeAttack),
    Knockout(Knockout),
    ReverseCupV2(ReverseCupV2),
}

impl ModeSettings {
    pub fn into_xml(&self) -> String {
        match self {
            ModeSettings::Rounds(rounds) => rounds.into_xml(),
            ModeSettings::ReverseCup(reverse_cup) => reverse_cup.into_xml(),
            ModeSettings::TimeAttack(time_attack) => time_attack.into_xml(),
            ModeSettings::Knockout(knockout) => knockout.into_xml(),
            ModeSettings::ReverseCupV2(reverse_cup_v2) => reverse_cup_v2.into_xml(),
        }
    }

    pub fn get_xml_map(&self) -> BTreeMap<String, dxr::Value> {
        match self {
            ModeSettings::Rounds(rounds) => rounds.get_xml_map(),
            ModeSettings::ReverseCup(reverse_cup) => reverse_cup.get_xml_map(),
            ModeSettings::TimeAttack(time_attack) => time_attack.get_xml_map(),
            ModeSettings::Knockout(knockout) => knockout.get_xml_map(),
            ModeSettings::ReverseCupV2(reverse_cup_v2) => reverse_cup_v2.get_xml_map(),
        }
    }

    pub fn mode_header(&self) -> String {
        match self {
            ModeSettings::Rounds(_) => {
                "<script_name>Trackmania/TM_Rounds_Online</script_name>".into()
            }
            ModeSettings::ReverseCup(_) => {
                "<script_name>Modes/Trackmania/ReverseCup</script_name>".into()
            }
            ModeSettings::TimeAttack(_) => {
                "<script_name>Trackmania/TM_TimeAttack_Online</script_name>".into()
            }
            ModeSettings::Knockout(_) => {
                "<script_name>Trackmania/TM_Knockout_Online</script_name>".into()
            }
            ModeSettings::ReverseCupV2(_) => {
                "<script_name>Modes/Trackmania/ReverseCup</script_name>".into()
            }
        }
    }

    pub fn script_name(&self) -> &str {
        match self {
            ModeSettings::Rounds(_) => "Trackmania/TM_Rounds_Online",
            ModeSettings::ReverseCup(_) => "Modes/Trackmania/ReverseCup",
            ModeSettings::TimeAttack(_) => "Trackmania/TM_TimeAttack_Online",
            ModeSettings::Knockout(_) => "Trackmania/TM_Kockout_Online",
            ModeSettings::ReverseCupV2(_) => "Modes/Trackmania/ReverseCup",
        }
    }

    pub fn get_mode(&self) -> TmMode {
        match self {
            ModeSettings::Rounds(_) => TmMode::Rounds,
            ModeSettings::ReverseCup(_) => TmMode::ReverseCup,
            ModeSettings::TimeAttack(_) => TmMode::TimeAttack,
            ModeSettings::Knockout(_) => TmMode::Knockout,
            ModeSettings::ReverseCupV2(_) => TmMode::ReverseCup,
        }
    }

    /// Returns the mode script of a custom mode.
    pub fn get_external_script(&self) -> Option<&str> {
        match self {
            ModeSettings::Rounds(_) => None,
            ModeSettings::ReverseCup(_) => {
                Some(include_str!("../external_modes/ReverseCup.Script.txt"))
            }
            ModeSettings::TimeAttack(_) => None,
            ModeSettings::Knockout(_) => None,
            ModeSettings::ReverseCupV2(_) => {
                Some(include_str!("../external_modes/ReverseCup.Script.txt"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct MapPoolConfig {
    start: u32,
    map_uids: Vec<String>,
}

impl MapPoolConfig {
    pub fn into_xml(&self) -> String {
        let start = format!(
            r#"
        <startindex>{}</startindex>
        "#,
            self.start
        );
        let mut maps = start;
        for map in &self.map_uids {
            maps += &format!("<map><file>{}.Map.Gbx</file></map>", map);
        }
        maps
    }

    pub fn maps(&self) -> Vec<String> {
        self.map_uids.clone()
    }
}

impl Default for MapPoolConfig {
    /// Playlist with Training01
    fn default() -> Self {
        Self {
            start: 0,
            map_uids: vec!["olsKnq_qAghcVAnEkoeUnVHFZei".into()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub enum TmMode {
    Rounds,
    ReverseCup,
    Knockout,
    TimeAttack,
    // This is used upstream in the server-manager module so its not super clean separation of types.
    Unknown,
}

fn tm_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}
