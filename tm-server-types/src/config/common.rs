/// The configuration available in every game mode.
/// Only usable parameters included (not shootmania stuff): [Docs](https://wiki.trackmania.io/en/dedicated-server/Usage/OfficialGameModesSettings#s_decoimageurl_checkpoint)
/// Omitted:
/// - Inifnte Laps: Reproducible with Force Laps Number
/// - Script Environment: No dev support
/// - Season Ids: Nobody knows what it does
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Common {
    /// Chat time at the end of a map or match
    chat_time: u32,
    respawn_behaviour: RespawnBavaviour,

    /// Synchronize players at the launch of the map, to ensure that no one starts late.
    /// Can delay the start by a few seconds.
    synchronize_players_at_map_start: bool,
    /// Synchronize players at the launch of the round, to ensure that no one starts late.
    /// Can delay the start by a few seconds.
    synchronize_players_at_round_start: bool,
    /// No clear official informations about this setting.
    /// It would seem that this tells the server to trust or not trust the network data sent by the client.
    trust_client_simulation: bool,

    /// The car position of other players is extrapolated less precisely, disabling it has a big impact on performance.
    /// This replaces the "S_UseDelayedVisuals" option by removing the delay with ghosts for the modes that need it (There may be a delay in TimeAttack).
    use_crude_extrapolation: bool,

    warmup_duration: WarmupDuration,
    //warmup_timeout: ,
    warmup_number: u32,

    /// Url of the image displayed on the checkpoints ground.
    /// Override the image set in the Club.
    deco_image_url_checkpoint: String,
    /// Url of the image displayed on the block border.
    /// Override the image set in the Club.
    deco_image_url_decal_sponsor_4x1: String,
    /// Url of the image displayed below the podium and big screen.
    /// Override the image set in the Club.
    deco_image_url_screen_16x1: String,
    /// Url of the image displayed on the two big screens.
    /// Override the image set in the Club.
    deco_image_url_screen_19x9: String,
    /// Url of the image displayed on the bleachers.
    /// Override the image set in the Club.
    deco_image_url_screen_8x1: String,
    /// Url of the API route to get the deco image url.
    /// You can replace ":ServerLogin" with a login from a server in another club to use its images.
    deco_image_url_who_am_i: String,

    force_laps_number: i32,
}

impl Common {
    pub fn default_rounds() -> Self {
        Self {
            chat_time: 10,
            respawn_behaviour: RespawnBavaviour::Default,
            synchronize_players_at_map_start: true,
            synchronize_players_at_round_start: true,
            trust_client_simulation: true,
            use_crude_extrapolation: true,
            warmup_duration: WarmupDuration::BasedOnMedal,
            warmup_number: 0,
            deco_image_url_checkpoint: "".into(),
            deco_image_url_decal_sponsor_4x1: "".into(),
            deco_image_url_screen_16x1: "".into(),
            deco_image_url_screen_19x9: "".into(),
            deco_image_url_screen_8x1: "".into(),
            deco_image_url_who_am_i: "".into(),
            force_laps_number: -1, // Laps from map validation
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub enum RespawnBavaviour {
    /// Use the default behavior of the gamemode
    Default = 0,
    /// Use the normal behavior like in TimeAttack.
    TimeAttack = 1,
    /// Do nothing.
    Ignore = 2,
    /// Give up before first checkpoint.
    GiveUpAtStart = 3,
    /// Always give up.
    GiveUpAlways = 4,
    /// Never give up.
    GiveUpNever = 5,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
#[cfg_attr(feature = "serde", serde(from = "i32", into = "i32"))]
pub enum WarmupDuration {
    /// Only one try like a round
    OneTry,
    // Time based on the Author medal ( 5 seconds + Author Time on 1 lap + ( Author Time on 1 lap / 6 ) )
    BasedOnMedal,
    /// Time in seconds
    Seconds(u32),
}

impl From<i32> for WarmupDuration {
    fn from(value: i32) -> Self {
        match value {
            -1 => WarmupDuration::OneTry,
            0 => WarmupDuration::BasedOnMedal,
            _ => WarmupDuration::Seconds(value as u32),
        }
    }
}

impl From<WarmupDuration> for i32 {
    fn from(value: WarmupDuration) -> Self {
        match value {
            WarmupDuration::OneTry => -1,
            WarmupDuration::BasedOnMedal => 0,
            WarmupDuration::Seconds(s) => s as i32,
        }
    }
}
