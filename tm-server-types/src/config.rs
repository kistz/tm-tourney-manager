mod rounds;
use std::fmt::format;

pub use rounds::Rounds;

mod common;
pub use common::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct ServerConfig {
    common: Common,
    mode: ModeConfig,
    //playlist: Playlist,
}

impl ServerConfig {
    pub fn get_common(&self) -> &Common {
        &self.common
    }

    pub fn get_mode(&self) -> &ModeConfig {
        &self.mode
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            common: Common::default_rounds(),
            mode: ModeConfig::Rounds(Rounds::default()),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub enum ModeConfig {
    Rounds(Rounds),
}

impl ModeConfig {
    pub fn get_settings(&self) -> String {
        match self {
            ModeConfig::Rounds(rounds) => format!(
                r#"
        <setting name="S_PointsLimit" value="{}" type="integer"/>
        <setting name="S_RoundsPerMap" value="{}" type="integer"/>
        <setting name="S_MapsPerMatch" value="{}" type="integer"/>
        <setting name="S_PointsRepartition" value="{:?}" type="text"/>
        <setting name="S_UseCustomPointsRepartition" value="{}" type="boolean"/>
        <setting name="S_DelayBeforeNextMap" value="{}" type="integer"/>
        <setting name="S_FinishTimeout" value="{}" type="integer"/>
            "#,
                rounds.points_limit,
                rounds.rounds_per_map,
                rounds.mpas_per_match,
                rounds.points_repartition,
                rounds.use_custom_points_repartition,
                rounds.delay_before_next_map,
                rounds.finish_timeout,
            ),
        }
    }
}
