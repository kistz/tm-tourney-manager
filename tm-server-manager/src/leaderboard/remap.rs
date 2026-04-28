use spacetimedb::SpacetimeType;

use crate::leaderboard::{LbManipulationKind, LbParams};

#[derive(Debug, SpacetimeType)]
pub(super) struct LbRemapSettings {
    kind: LbRemapKind,
    param: LbParams,
    manipulation: LbManipulationKind,
    // TODO should this be a f32???
    manipulation_value: i32,
}

#[derive(Debug, SpacetimeType)]
enum LbRemapKind {
    Match,
    Rounds,
}
