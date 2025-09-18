#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct WayPointEvent {
    #[cfg_attr(feature = "serde", serde(rename = "accountid"))]
    account_id: String,
    login: String,
    time: u32,
    racetime: u32,
    laptime: u32,
    speed: f32,

    #[cfg_attr(feature = "serde", serde(rename = "checkpointinrace"))]
    checkpoint_in_race: u32,
    #[cfg_attr(feature = "serde", serde(rename = "checkpointinlap"))]
    checkpoint_in_lap: u32,
    #[cfg_attr(feature = "serde", serde(rename = "isendrace"))]
    is_end_race: bool,
    #[cfg_attr(feature = "serde", serde(rename = "isendlap"))]
    is_end_lap: bool,
    #[cfg_attr(feature = "serde", serde(rename = "isinfinitelaps"))]
    is_infinite_laps: bool,
    #[cfg_attr(feature = "serde", serde(rename = "isindependentlaps"))]
    is_independent_laps: bool,
    #[cfg_attr(feature = "serde", serde(rename = "curracecheckpoints"))]
    current_race_checkpoints: Vec<u32>,
    #[cfg_attr(feature = "serde", serde(rename = "curlapcheckpoints"))]
    current_lap_checkpoints: Vec<u32>,
    #[cfg_attr(feature = "serde", serde(rename = "blockid"))]
    block_id: String,
}
