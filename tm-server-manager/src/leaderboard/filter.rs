use std::collections::{HashMap, HashSet};

use spacetimedb::SpacetimeType;

use crate::{
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
}

impl LbFilterSettings {
    pub(super) fn evaluate(self, leaderboard: Vec<LbEntry>) -> Vec<LbEntry> {
        let mut map: HashMap<u32, Vec<LbEntry>> = HashMap::new();

        for row in leaderboard {
            map.entry(row.user_id)
                .and_modify(|e| {
                    e.push(row);
                })
                .or_insert(vec![row]);
        }

        for rows in map.values_mut() {
            match self.kind {
                //LbFilterKind::Match => todo!(),
                //LbFilterKind::Maps => todo!(),
                //TODO valiidate that the sort does all edge cases.
                LbFilterKind::Rounds => rows.sort_by(|a, b| match self.param {
                    LbParams::Score => a.score.cmp(&b.score),
                    // e.g. -43 seconds is more than -44 seconds.
                    // because otherwise 43 would be less than 44
                    // but less is better so we need invert.
                    LbParams::Time => (-a.time).cmp(&(-b.time)),
                    // Same reson as above. A lower position is better.
                    LbParams::Position => b.position.cmp(&a.position),
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

// separate by map
// for each map filter rounds best 5/6 merge rounds averge position

// match is match or leadarboard or probably also monitoring in the future.
// how to incorporate this? map_id: u32, | for match its pf for leadaerboard its harder.
