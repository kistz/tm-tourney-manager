use std::collections::HashMap;

use spacetimedb::{AnonymousViewContext, Query, SpacetimeType, table, view};
use tm_server_types::config::{ModeSettings, TmMode};

use crate::{
    raw_server::{config::RawServerContigRead, occupation::TabRawServerOccupationRead},
    tm_match::{state::tab_match_state__view, tab_match__view},
};

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) enum PlayerAction {
    StartLine(u32),
    Checkpoint(PlayerActionCheckpoint),
    Respawn(PlayerActionRespawn),
    GiveUp(u32),
    Lap(PlayerActionCheckpoint),
    Finish(PlayerActionCheckpoint),
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct PlayerActionRespawn {
    time: u32,
    speed: f32,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub(super) struct PlayerActionCheckpoint {
    speed: f32,
    time: u32,
}

#[table(accessor= tab_match_round_player,
    index(accessor=match_round, hash(columns=[match_id,round])),
    index(accessor=match_round_range, btree(columns=[match_id,round])),
    index(accessor=match_round_player, hash(columns=[match_id,round,user_id]))
)]
#[derive(Debug, Clone, Copy)]
pub struct MatchRoundPlayer {
    #[auto_inc]
    #[primary_key]
    pub id: u32,

    #[index(hash)]
    pub user_id: u32,

    #[index(hash)]
    match_id: u32,
    time: i32,
    // The points of the round.
    points: i32,

    round: u16,
    #[default(0)]
    position: u16,
}

impl MatchRoundPlayer {
    pub(super) fn new(match_id: u32, user_id: u32, round: u16) -> Self {
        Self {
            user_id,
            match_id,
            round,
            time: 0,
            points: 0,
            id: 0,
            position: 0,
        }
    }

    pub(super) fn set_time(&mut self, points: i32) {
        self.time = points;
    }

    pub(super) fn set_points(&mut self, points: i32) {
        self.points = points;
    }

    pub(super) fn set_position(&mut self, position: u16) {
        self.position = position;
    }

    pub(crate) fn get_time(&self) -> i32 {
        self.time
    }

    pub(crate) fn get_position(&self) -> u16 {
        self.position
    }

    pub(crate) fn get_score(&self) -> i32 {
        self.points
    }
}

#[table(accessor= tab_match_round_player_ext,
    index(accessor=match_round, hash(columns=[match_id,round])),
    index(accessor=match_round_range, btree(columns=[match_id,round])),
    index(accessor=match_round_player, hash(columns=[match_id,round,user_id]))
)]
pub struct MatchRoundPlayerExt {
    round_actions: Vec<PlayerAction>,

    user_id: u32,
    #[primary_key]
    pub id: u32,
    #[index(hash)]
    match_id: u32,
    round: u16,
}

impl MatchRoundPlayerExt {
    pub fn new(id: u32, match_id: u32, user_id: u32, round: u16, server_time: u32) -> Self {
        Self {
            user_id,
            match_id,
            round,
            round_actions: vec![PlayerAction::StartLine(server_time)],
            id,
        }
    }

    pub(crate) fn add_checkpoint(&mut self, speed: f32, time: u32) {
        self.round_actions
            .push(PlayerAction::Checkpoint(PlayerActionCheckpoint {
                speed,
                time,
            }));
    }
    pub(crate) fn add_lap(&mut self, speed: f32, time: u32) {
        self.round_actions
            .push(PlayerAction::Lap(PlayerActionCheckpoint { speed, time }));
    }
    pub(crate) fn add_finish(&mut self, speed: f32, time: u32) {
        self.round_actions
            .push(PlayerAction::Finish(PlayerActionCheckpoint { speed, time }));
    }

    pub(crate) fn add_respawn(&mut self, speed: f32, server_time: u32) {
        let first = *self.round_actions.first().unwrap();

        // Double respawn.
        if speed == 0.
            && let Some(last) = self.round_actions.last_mut()
            && let PlayerAction::Respawn(respawn) = last
        {
            respawn.speed = speed;

            return;
        };

        if let PlayerAction::StartLine(time) = first {
            self.round_actions
                .push(PlayerAction::Respawn(PlayerActionRespawn {
                    speed,
                    time: server_time - time,
                }));
        } else {
            log::error!("First event in a RoundAction was something other than start line event.")
        }
    }

    pub(crate) fn give_up(&mut self, server_time: u32) {
        let first = self.round_actions.first().unwrap();

        if let PlayerAction::StartLine(time) = *first {
            self.round_actions
                .push(PlayerAction::GiveUp(server_time - time));
        } else {
            log::error!("First event in a RoundAction was something other than start line event.")
        }
    }
}

#[view(accessor=temp_match_leaderboard,public)]
fn temp_match_leaderboard(
    ctx: &AnonymousViewContext, /*, match_id: u32, round: u16 */
) -> Vec<MatchRoundPlayer> {
    ctx.match_leaderboard(51, 0)
}

/// Returns the specified round of the match.
/// Round 0 is giving you a live view.
/// If you want a accumulated view please you the match_leaderboard view instead.
#[view(accessor=match_round,public)]
fn match_round(
    ctx: &AnonymousViewContext, /*, match_id: u32, round: u16 */
) -> Vec<MatchRoundPlayer> {
    let match_id = 51u32;
    let mut round = 0u16;

    if round == 0 {
        let Some(state) = ctx.db.tab_match_state().match_id().find(match_id) else {
            return Vec::new();
        };
        round = state.get_round();
    }

    let mut standings = ctx
        .db
        .tab_match_round_player()
        .match_round()
        .filter((match_id, round))
        .collect::<Vec<_>>();
    // This is part of the contracft of the function!!!
    // For calls in the module e.g. depending nodes requesting results. it needs to be sorted correctly.
    standings.sort_by_key(|v| v.points);
    standings
}

#[view(accessor=unstable_match_round,public)]
fn unstable_match_round(
    ctx: &AnonymousViewContext, /*, match_id: u32, round: u16 */
) -> impl Query<MatchRoundPlayer> {
    ctx.from.tab_match_round_player()
}

#[view(accessor=unstable_match_round_ext,public)]
fn unstable_match_round_ext(
    ctx: &AnonymousViewContext, /*, match_id: u32, round: u16 */
) -> impl Query<MatchRoundPlayerExt> {
    ctx.from.tab_match_round_player_ext()
}

/// If round 0 is supplied we take the current round.
#[view(accessor=match_round_ext,public)]
fn match_round_ext(
    ctx: &AnonymousViewContext, /* match_id: u32, round: u16 */
) -> Vec<MatchRoundPlayerExt> {
    let match_id = 51u32;
    let mut round = 0u16;

    if round == 0 {
        let Some(state) = ctx.db.tab_match_state().match_id().find(match_id) else {
            return Vec::new();
        };
        round = state.get_round();
    }

    ctx.db
        .tab_match_round_player_ext()
        .match_round()
        .filter((match_id, round))
        .collect()
}

pub(crate) trait MatchLeadearboardRead {
    fn match_leaderboard(&self, match_id: u32, round: u16) -> Vec<MatchRoundPlayer>;
    fn match_rounds(&self, match_id: u32) -> Vec<MatchRoundPlayer>;
}
impl<Db: spacetimedb::DbContext> MatchLeadearboardRead for Db {
    /// Accumulates points of all previous rounds.
    /// Round 0 is giving you a live view.
    /// If you want points from individual rounds use the match_round view instead.
    /// # Important.
    /// Returns a SORTED VECTOR of the standings of the mode.
    /// This is part of the contract and MUST not be changed.
    fn match_leaderboard(&self, match_id: u32, mut round: u16) -> Vec<MatchRoundPlayer> {
        let Some(state) = self
            .db_read_only()
            .tab_match_state()
            .match_id()
            .find(match_id)
        else {
            return Vec::new();
        };
        if round == 0 {
            round = state.get_round();
        };
        let mut entries: Vec<MatchRoundPlayer> = self
            .db_read_only()
            .tab_match_round_player()
            .match_round_range()
            .filter((match_id, 1..=round))
            .collect();

        let mut map = HashMap::<u32, MatchRoundPlayer>::new();

        let returned = match state.get_mode() {
            TmMode::Rounds => {
                for entry in entries {
                    map.entry(entry.user_id)
                        .and_modify(|e| {
                            e.points += entry.points;
                            if entry.round > e.round {
                                e.round = entry.round;
                                e.id = entry.id;
                            }
                        })
                        .or_insert(entry);
                }

                let mut standings = map.into_values().collect::<Vec<_>>();

                standings.sort_by_key(|v| -v.points);
                standings
            }
            TmMode::ReverseCup => {
                for entry in entries {
                    map.entry(entry.user_id)
                        .and_modify(|e| {
                            e.points += entry.points;
                            if entry.round > e.round {
                                e.round = entry.round;
                                e.id = entry.id;
                            }
                        })
                        .or_insert(entry);
                }

                let mut standings = map.into_values().collect::<Vec<_>>();

                let tm_match = self.db_read_only().tab_match().id().find(match_id).unwrap();
                let cfg = self.raw_server_config(tm_match.config).unwrap();
                let starting_points = match cfg.get_mode() {
                    ModeSettings::ReverseCup(reverse_cup) => reverse_cup.starting_points,
                    _ => unreachable!(),
                };

                for player in &mut standings {
                    player.points += starting_points;
                }

                //TODO this is wildly incorrect.
                standings.sort_by_key(|v| v.points);
                standings
            }
            TmMode::Knockout => {
                for entry in entries {
                    map.entry(entry.user_id)
                        .and_modify(|e| {
                            if entry.points == -2 {
                                *e = entry;
                            }
                        })
                        .or_insert(entry);
                }
                let mut standings = map.into_values().collect::<Vec<_>>();

                standings.sort_by_key(|v| v.round);
                standings
            }
            TmMode::TimeAttack => {
                entries.sort_by_key(|v| if v.time == 0 { i32::MAX } else { v.time });

                entries
            }
        };

        log::info!("{returned:?}");

        returned
    }

    fn match_rounds(&self, match_id: u32) -> Vec<MatchRoundPlayer> {
        self.db_read_only()
            .tab_match_round_player()
            .match_id()
            .filter(match_id)
            .collect()
    }
}
