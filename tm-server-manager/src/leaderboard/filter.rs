use std::collections::{HashMap, HashSet};

use spacetimedb::{DbContext, SpacetimeType};
use tm_server_types::config::TmMode;

use crate::{
    competition::node::{NodeHandle, NodeLeaderboard},
    leaderboard::{LbEntry, LbParams},
    tm_match::leaderboard::MatchRoundPlayer,
};

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct LbFilterSettings {
    kind: LbFilterKind,
    param: LbParams,
    filter: LbFilterIdent,
    fallback: LbFilterFallback,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbFilterIdent {
    Best(u16),
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbFilterFallback {
    Worst,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbFilterKind {
    //Maps,
    Rounds,
    // Actions // TODO we should be able to inspect the actions of the player.
    Matches,
}

impl LbFilterSettings {
    pub(super) fn evaluate(
        self,
        lb_id: u32,
        leaderboard: Vec<LbEntry>,
        ctx: &impl DbContext,
    ) -> Vec<LbEntry> {
        let mut map: HashMap<u32, Vec<LbEntry>> = HashMap::new();

        match self.kind {
            //LbFilterKind::Match => todo!(),
            //LbFilterKind::Maps => todo!(),
            //TODO valiidate that the sort does all edge cases.
            LbFilterKind::Rounds =>
            /* rows.sort_by(|a, b| match self.param {
                LbParams::Score => a.score.cmp(&b.score),
                // e.g. -43 seconds is more than -44 seconds.
                // because otherwise 43 would be less than 44
                // but less is better so we need invert.
                LbParams::Time => (-a.time).cmp(&(-b.time)),
                // Same reson as above. A lower position is better.
                LbParams::Position => b.position.cmp(&a.position),
            }) */
            {
                for row in leaderboard {
                    map.entry(row.user_id)
                        .and_modify(|e| {
                            e.push(row);
                        })
                        .or_insert(vec![row]);
                }
            }
            LbFilterKind::Matches => {
                let mut lb_nodes: HashMap<NodeHandle, Vec<LbEntry>> = HashMap::new();
                for row in leaderboard {
                    lb_nodes
                        .entry(row.get_node())
                        .and_modify(|e| {
                            e.push(row)
                            /* e.entry(row.get_user())
                            .and_modify(|e| e.push(row))
                            .or_insert(vec![row]); */
                        })
                        .or_insert(vec![row]);
                }

                //let mut new_map: HashMap<u32, Vec<LbEntry>> = HashMap::new();
                for lb_entires in lb_nodes.into_values() {
                    let lb_result = lb_entires.finalize(ctx);
                    for user_result in lb_result {
                        map.entry(user_result.get_user())
                            .and_modify(|e| e.push(user_result))
                            .or_insert(vec![user_result]);
                    }
                }
            }
        }

        let map_len = map.len();

        if map_len == 0 {
            return Vec::new();
        }

        for rows in map.values_mut() {
            match self.param {
                LbParams::Score => {
                    rows.sort_by_key(|f| -f.score);
                }
                LbParams::Time => rows.sort_by_key(|f| if f.time <= 0 { i32::MAX } else { f.time }),
                LbParams::Position => rows.sort_by_key(|f| f.position),
            }
        }

        //TODO this is not possible because matches need to query it differently.
        let Some(max_rounds) = map.values().max_by_key(|v| v.len()) else {
            // This case is only hit when nobody is even there so it is safe to just return nothing.
            return Vec::new();
        };
        let max_rounds = max_rounds.len() as u16;

        for (user_id, rows) in &mut map {
            match self.filter {
                LbFilterIdent::Best(n) => {
                    rows.truncate(n as usize);
                    if rows.len() < (n as usize) {
                        // if we have played less rounds than the best n we take the current row count.
                        let needed = if n > max_rounds { max_rounds } else { n };
                        let missing = needed - rows.len() as u16;
                        match self.fallback {
                            LbFilterFallback::Worst => match self.param {
                                LbParams::Score => {
                                    for _ in 0..missing {
                                        //TODO consider mode
                                        rows.push(
                                            LbEntry::new(
                                                *user_id,
                                                TmMode::Unknown,
                                                map_len as u16,
                                                NodeHandle::LeaderboardV1(lb_id),
                                            )
                                            .set_score(0),
                                        );
                                    }
                                }
                                LbParams::Time => (), //TODO
                                LbParams::Position => {
                                    for _ in 0..missing {
                                        rows.push(LbEntry::new(
                                            *user_id,
                                            TmMode::Unknown,
                                            map_len as u16,
                                            NodeHandle::LeaderboardV1(lb_id),
                                        ));
                                    }
                                }
                            },
                        }
                    }
                }
            };
        }

        let filter = map.into_values().flatten().collect();
        log::info!("Filter Result: {:?}", filter);
        filter
    }
}

// filter per: (everything specific to user) | think it the other way around? if i request a filter rounds we automatically separate by maps?
// matches -> outputs match | invalid for filter
// match -> outputs maps | maps can either be map count or map id i guess e.g. if map is played 2 times.
// map -> outputs rounds
// rounds -> outputs params

// Input: its always the MatchRoundPlayer.
// that means that we always sort by player???????

// keyed/ unkeyed filter?
// a big problem are the orphaned fields which proceed to be meaningless when remapping stuff 🤔
// position should always be recomputed i guess
// the match is a problem when propagating... this is because the leadearboard is now also something which is able to be searched for 🤔
// this means the connection should give me the leaderboard and then we also shoud do a new struct i reckon.

// separate by map
// for each map filter rounds best 5/6 merge rounds averge position

// match is match or leadarboard or probably also monitoring in the future.
// how to incorporate this? map_id: u32, | for match its pf for leaderboard its harder.
