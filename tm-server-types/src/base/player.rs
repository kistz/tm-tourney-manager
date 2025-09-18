#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Player {
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
