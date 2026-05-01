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
    //Summate,
}

impl LbMergeSettings {
    pub(super) fn evaluate(self, leaderboard: Vec<LbEntry>) -> Vec<LbEntry> {
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
                                    //let float = accumulated.score as f32 / num_rounds as f32;
                                    let avg = (accumulated.score as f32 / num_rounds as f32) as i32;
                                    accumulated.score = avg;

                                    map.insert(player, accumulated);
                                }
                                LbParams::Time => {
                                    let mut accumulated =
                                        rounds.into_iter().fold(thing, |mut acc, x| {
                                            acc.time += x.time;
                                            acc
                                        });
                                    //TODO make float var sort with that and then cast abck to int.
                                    //let float = accumulated.score as f32 / num_rounds as f32;
                                    let avg = (accumulated.time as f32 / num_rounds as f32) as i32;
                                    accumulated.time = avg;

                                    map.insert(player, accumulated);
                                }
                                LbParams::Position => {
                                    let mut accumulated =
                                        rounds.into_iter().fold(thing, |mut acc, x| {
                                            acc.position += x.position;
                                            acc
                                        });
                                    //TODO make float var sort with that and then cast abck to int.
                                    //let float = accumulated.score as f32 / num_rounds as f32;
                                    let avg =
                                        (accumulated.position as f32 / num_rounds as f32) as u16;
                                    accumulated.position = avg;

                                    map.insert(player, accumulated);
                                }
                            }
                        }
                        match self.param {
                            LbParams::Score => {
                                let mut vec = map.into_values().collect::<Vec<_>>();
                                vec.sort_by_key(|f| f.score);
                                // readjust the position.
                                vec.iter_mut()
                                    .enumerate()
                                    .for_each(|(i, e)| e.position = (i + 1) as u16);
                                vec
                            }
                            LbParams::Time => {
                                let mut vec = map.into_values().collect::<Vec<_>>();
                                vec.sort_by_key(|f| f.time);

                                vec
                            }
                            LbParams::Position => {
                                let mut vec = map.into_values().collect::<Vec<_>>();
                                vec.sort_by_key(|f| f.position);

                                vec
                            }
                        }
                    } //LbMergeAction::Summate => todo!(),
                }
            } //LbMergeKind::Matches => {}
        }
    }
}
