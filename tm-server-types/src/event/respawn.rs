#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Respawn {
    #[cfg_attr(feature = "serde", serde(rename = "accountid"))]
    account_id: String,
    login: String,
    time: u32,

    #[cfg_attr(feature = "serde", serde(rename = "nbrespawns"))]
    number_respawns: u32,

    racetime: i32,
    laptime: i32,

    #[cfg_attr(feature = "serde", serde(rename = "checkpointinrace"))]
    checkpoint_in_race: i32,

    #[cfg_attr(feature = "serde", serde(rename = "checkpointinlap"))]
    checkpoint_in_lap: i32,

    speed: f32,
}
