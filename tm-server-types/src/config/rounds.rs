#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
struct Rounds {
    chat_time: u32,
    delay_before_next_map: u32,
    finish_timeout: u32,
    force_laps_number: i32,
    infinite_laps: bool,
    is_channel_server: bool,
    is_split_screen: bool,
    mpas_per_match: u32,
    points_limit: u32,
    points_repartition: Vec<u32>,
    respawn_behaviour: u32,
    rounds_per_map: u32,
    warm_up_duration: u32,
    warm_up_number: u32,
}

impl Default for Rounds {
    fn default() -> Self {
        Self {
            chat_time: Default::default(),
            delay_before_next_map: Default::default(),
            finish_timeout: Default::default(),
            force_laps_number: Default::default(),
            infinite_laps: Default::default(),
            is_channel_server: Default::default(),
            is_split_screen: Default::default(),
            mpas_per_match: Default::default(),
            points_limit: Default::default(),
            points_repartition: Default::default(),
            respawn_behaviour: Default::default(),
            rounds_per_map: Default::default(),
            warm_up_duration: Default::default(),
            warm_up_number: Default::default(),
        }
    }
}
