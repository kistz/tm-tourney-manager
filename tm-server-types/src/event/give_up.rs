#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct GiveUp {
    #[cfg_attr(feature = "serde", serde(rename = "accountid"))]
    account_id: String,
    login: String,
    time: u32,
}
