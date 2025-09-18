use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RoundsConfig {
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

impl RoundsConfig {
    /* fn new() -> Self {
        RoundsConfig {
            chat_time: 10,
            delay_before_next_map: (),
            finish_timeout: (),
            force_laps_number: (),
            infinite_laps: (),
            is_channel_server: (),
            is_split_screen: (),
            mpas_per_match: 0,
            points_limit: (),
            points_repartition: (),
            respawn_behaviour: (),
            rounds_per_map: 1,
            warm_up_number: 0,
            warm_up_duration: 60,
        }
    } */
}
