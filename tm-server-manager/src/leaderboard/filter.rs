use std::collections::{HashMap, HashSet};

use spacetimedb::SpacetimeType;

use crate::{leaderboard::LbParams, tm_match::leaderboard::MatchRoundPlayer};

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct LbFilterSettings {
    kind: LbFilterKind,
    param: LbParams,
    filter: LbFilterIdent,
    //manipulation: LbManipulationKind,
    // TODO should this be a f32???
    //manipulation_value: i32,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbFilterIdent {
    Best(u16),
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbFilterKind {
    Match,
    Maps,
    Rounds,
    // Actions // TODO we should be able to inspect the actions of the player.
}

impl LbFilterSettings {
    pub(super) fn evaluate(self, leaderboard: Vec<MatchRoundPlayer>) -> Vec<MatchRoundPlayer> {
        let mut map: HashMap<u32, Vec<MatchRoundPlayer>> = HashMap::new();

        for row in leaderboard {
            map.entry(row.user_id)
                .and_modify(|e| {
                    e.push(row);
                })
                .or_insert(vec![row]);
        }

        for rows in map.values_mut() {
            match self.kind {
                LbFilterKind::Match => todo!(),
                LbFilterKind::Maps => todo!(),
                //TODO valiidate that the sort does all edge cases.
                LbFilterKind::Rounds => rows.sort_by(|a, b| match self.param {
                    // Inverted because more score = better.
                    LbParams::Score => (-a.get_score()).cmp(&(-b.get_score())),
                    LbParams::Time => a.get_time().cmp(&b.get_time()),
                    LbParams::Position => a.get_position().cmp(&b.get_position()),
                }),
            }

            match self.filter {
                LbFilterIdent::Best(n) => rows.truncate(n as usize),
            };
        }

        map.into_values().flatten().collect()
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
