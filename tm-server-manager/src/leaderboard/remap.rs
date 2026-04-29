use spacetimedb::SpacetimeType;

use crate::leaderboard::LbParams;

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct LbRemapSettings {
    kind: LbRemapKind,
    param: LbParams,
    manipulation: LbManipulationKind,
    // TODO should this be a f32???
    manipulation_value: i32,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbRemapKind {
    Match,
    Rounds,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbManipulationKind {
    Subtract,
    Add,
    Multiply,
    Divide,
}
