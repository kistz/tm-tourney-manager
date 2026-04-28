use spacetimedb::SpacetimeType;

#[derive(Debug, SpacetimeType)]
pub(super) struct LbFilterSettings {
    /* kind: LbRemapKind,
    param: LbParams,
    manipulation: LbManipulationKind,
    // TODO should this be a f32???
    manipulation_value: i32, */
}

#[derive(Debug, SpacetimeType)]
enum LbRemapKind {
    Match,
    Rounds,
}
