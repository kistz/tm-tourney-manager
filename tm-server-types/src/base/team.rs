#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Team {
    id: u32,
    name: String,
    #[serde(rename = "roundpoints")]
    round_points: u32,
    #[serde(rename = "mappoints")]
    map_points: u32,
    #[serde(rename = "matchpoints")]
    match_points: u32,
}
