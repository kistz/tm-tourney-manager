use core::num;
use std::collections::HashMap;

use spacetimedb::SpacetimeType;
use tm_server_types::config::TmMode;

use crate::{
    competition::node::NodeHandle,
    leaderboard::{LbEntry, LbParams},
    tm_match::leaderboard::MatchRoundPlayer,
};

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct LbMergeSettings {
    /// This whole thing should be considered deprecated it has no meaning i think.
    kind: LbMergeKind,
    action: LbMergeAction,
    param: LbParams,
}

/// This whole thing should be considered deprecated it has no meaning i think.
#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbMergeKind {
    Rounds,
}

// Is this only for collisions?
#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbMergeAction {
    Average,
    Summate,
}

impl LbMergeSettings {
    pub(super) fn evaluate(self, lb_id: u32, leaderboard: Vec<LbEntry>) -> Vec<LbEntry> {
        let mut player_rounds: HashMap<u32, Vec<LbEntry>> = HashMap::new();

        for row in leaderboard {
            player_rounds
                .entry(row.user_id)
                .and_modify(|e| {
                    e.push(row);
                })
                .or_insert(vec![row]);
        }

        let mut map: HashMap<u32, LbEntry> = HashMap::new();
        match self.action {
            LbMergeAction::Average => {
                for (player, rounds) in player_rounds {
                    let num_rounds = rounds.len();
                    //TODO set right position for this whole thing.
                    let init =
                        LbEntry::new(player, TmMode::Unknown, 0, NodeHandle::LeaderboardV1(lb_id))
                            .set_score(0)
                            .set_time(0);
                    let mut accumulated =
                        rounds
                            .into_iter()
                            .fold(init, |mut acc, x| match self.param {
                                LbParams::Score => {
                                    acc.score += x.score;
                                    acc
                                }

                                LbParams::Time => {
                                    acc.time += x.time;
                                    acc
                                }

                                LbParams::Position => {
                                    acc.position += x.position;
                                    acc
                                }
                            });
                    let avg = (accumulated.score as f32 / num_rounds as f32) as i32;
                    accumulated.score = avg;

                    map.insert(player, accumulated);
                }
            }
            LbMergeAction::Summate => {
                for (player, rounds) in player_rounds {
                    let init =
                        LbEntry::new(player, TmMode::Unknown, 0, NodeHandle::LeaderboardV1(lb_id))
                            .set_score(0)
                            .set_time(0);
                    let accumulated =
                        rounds
                            .into_iter()
                            .fold(init, |mut acc, x| match self.param {
                                LbParams::Score => {
                                    acc.score += x.score;
                                    acc
                                }
                                LbParams::Time => {
                                    acc.time += x.time;
                                    acc
                                }
                                LbParams::Position => {
                                    acc.position += x.position;
                                    acc
                                }
                            });
                    map.insert(player, accumulated);
                }
            }
        }
        log::info!("Merge Map Result: {:?}", map);

        let mut vec = map.into_values().collect::<Vec<_>>();
        match self.param {
            LbParams::Score => {
                vec.sort_by_key(|f| -f.score);
                // readjust the position.
                vec.iter_mut()
                    .enumerate()
                    .for_each(|(i, e)| e.position = (i + 1) as u16);
            }
            LbParams::Time => {
                vec.sort_by_key(|f| if f.time <= 0 { i32::MAX } else { f.time });
            }
            LbParams::Position => {
                vec.sort_by_key(|f| f.position);
            }
        }

        vec
    }
}
