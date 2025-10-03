#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Rounds {
    /// Minimal time before the server go to the next map in milliseconds.
    pub delay_before_next_map: u32,
    pub finish_timeout: i32,
    pub mpas_per_match: i32,
    pub points_limit: u32,
    pub use_custom_points_repartition: bool,
    pub points_repartition: Vec<u32>,
    pub rounds_per_map: i32,
}

impl Default for Rounds {
    fn default() -> Self {
        Self {
            delay_before_next_map: 2000,
            finish_timeout: -1,
            mpas_per_match: -1,
            points_limit: 50,
            points_repartition: vec![10, 6, 4, 3, 2, 1],
            rounds_per_map: -1,
            use_custom_points_repartition: false,
        }
    }
}
