use core::num;
use std::collections::HashMap;

use spacetimedb::SpacetimeType;

use crate::{
    leaderboard::{LbEntry, LbParams},
    tm_match::leaderboard::MatchRoundPlayer,
};

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct LbMergeSettings {
    kind: LbMergeKind,
    action: LbMergeAction,
    param: LbParams,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbMergeKind {
    //Matches, // these are multiple input connections???
    //Maps,
    Rounds,
}

// Is this only for collisions?
#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbMergeAction {
    Average,
    Summate,
}

impl LbMergeSettings {
    pub(super) fn evaluate(self, leaderboard: Vec<LbEntry>) -> Vec<LbEntry> {
        let iter = leaderboard.into_iter();

        /* let mut output = Vec::new();

        // Player<Matches<Maps<Rounds<MatchRoundPlayer>>>>
        let map: HashMap<u32, Vec<Vec<Vec<Vec<MatchRoundPlayer>>>>> = HashMap::new(); */
        match self.kind {
            LbMergeKind::Rounds => {
                let mut player_rounds: HashMap<u32, Vec<LbEntry>> = HashMap::new();

                for row in leaderboard {
                    player_rounds
                        .entry(row.user_id)
                        .and_modify(|e| {
                            e.push(row);
                        })
                        .or_insert(vec![row]);
                }

                match self.action {
                    LbMergeAction::Average => {
                        let mut map: HashMap<u32, LbEntry> = HashMap::new();
                        for (player, rounds) in player_rounds {
                            let num_rounds = rounds.len();
                            let thing = LbEntry::new(player);
                            match self.param {
                                LbParams::Score => {
                                    let mut accumulated =
                                        rounds.into_iter().fold(thing, |mut acc, x| {
                                            acc.score += x.score;
                                            acc
                                        });
                                        //TODO make float var sort with that and then cast abck to int.
                                        let accumulated.score as
                                    accumulated.score = (accumulated.score / num_rounds as );
                                    map.insert(player, accumulated);
                                }
                                LbParams::Time => todo!(),
                                LbParams::Position => todo!(),
                            }
                        }
                        match self.param {
                            LbParams::Score => {
                                let mut vec = map.into_values().into_iter().collect::<Vec<_>>();
                                vec.sort_by_key(|f| f.score);
                                vec
                            }
                            LbParams::Time => todo!(),
                            LbParams::Position => todo!(),
                        }
                    }
                    LbMergeAction::Summate => todo!(),
                }
            } /* LbMergeKind::Matches => {

              }, */
        }
    }
}
