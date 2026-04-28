use spacetimedb::SpacetimeType;

use crate::leaderboard::LbParams;

#[derive(Debug, SpacetimeType)]
pub(super) struct LbMergeSettings {
    kind: LbMergeKind,
    action: LbMergeAction,
    param: LbParams,
}

#[derive(Debug, SpacetimeType)]
enum LbMergeKind {
    //Matches, // these are multiple input connections???
    Rounds,
}

#[derive(Debug, SpacetimeType)]
enum LbMergeAction {
    //Matches, // these are multiple input connections???
    Average,
}
