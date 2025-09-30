use spacetimedb::{ReducerContext, Table, reducer, table};

use crate::{event::tournament_event, leaderboard::Leaderboard};

/// Each Event can have multiple stages association with it.
/// These are walked through _sequentially_ as the Event progresses.
/// This allows you to depend on the outcome of previous stages to determine players for the next stage.
#[table(name = event_stage,public)]
pub struct EventStage {
    #[auto_inc]
    #[primary_key]
    pub id: u128,

    event_id: u128,

    // Unique for the Event
    name: String,

    /// Matches get executed in parallel
    matches: Vec<u128>,
    //leaderboard: Leaderboard,
}

impl EventStage {
    pub fn add_match(&mut self, stage_match: u128) {
        self.matches.push(stage_match);
    }
}

/// Adds a new EventStage to the specified TournamentEvent.
#[reducer]
pub fn add_stage(ctx: &ReducerContext, name: String, to: u128, with: Option<u128>) {
    //TODO authorization
    if let Some(mut event) = ctx.db.tournament_event().id().find(to) {
        let stage = ctx.db.event_stage().insert(EventStage {
            id: 0,
            event_id: to,
            name,
            matches: Vec::new(),
        });

        event.add_stage(stage.id);

        ctx.db.tournament_event().id().update(event);
    }
}

#[table(name = stage_template,public)]
pub struct StageTemplate {}
