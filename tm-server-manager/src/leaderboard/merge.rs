use std::collections::HashMap;

use spacetimedb::SpacetimeType;

use crate::{leaderboard::LbParams, tm_match::leaderboard::MatchRoundPlayer};

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct LbMergeSettings {
    kind: LbMergeKind,
    action: LbMergeAction,
    param: LbParams,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbMergeKind {
    Matches, // these are multiple input connections???
    Maps,
    Rounds,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbMergeAction {
    Average,
    Summate,
}

impl LbMergeSettings {
    pub(super) fn evaluate(self, leaderboard: Vec<MatchRoundPlayer>) -> Vec<MatchRoundPlayer> {
        let iter = leaderboard.into_iter();

        let mut output = Vec::new();

        // Player<Matches<Maps<Rounds<MatchRoundPlayer>>>>
        let map: HashMap<u32, Vec<Vec<Vec<Vec<MatchRoundPlayer>>>>> = HashMap::new();

        match self.kind {
            LbMergeKind::Matches => todo!(), //Match leaderboard
            LbMergeKind::Maps => todo!(),    //Map leaderboard
            LbMergeKind::Rounds => iter,
        }

        output
    }
}
