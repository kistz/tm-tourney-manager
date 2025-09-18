#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Team {
    id: u32,
    name: String,
    #[cfg_attr(feature = "serde", serde(rename = "roundpoints"))]
    round_points: u32,
    #[cfg_attr(feature = "serde", serde(rename = "mappoints"))]
    map_points: u32,
    #[cfg_attr(feature = "serde", serde(rename = "matchpoints"))]
    match_points: u32,
}
