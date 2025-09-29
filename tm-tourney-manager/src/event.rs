use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};

use crate::{leaderboard::Leaderboard, tournament::tournament};

mod scheduling;

#[table(name = tournament_event,public)]
pub struct TournamentEvent {
    #[auto_inc]
    #[primary_key]
    pub id: u128,

    tournament: u128,

    // Unique for the tournament
    name: String,

    //TODO registered players

    //Scheduled time
    //starting: String,
    status: EventStatus,

    stages: Vec<u128>,
    //leaderboard: Leaderboard,
}

impl TournamentEvent {
    pub fn add_stage(&mut self, stage: u128) {
        self.stages.push(stage);
    }
}

#[derive(Debug, SpacetimeType)]
pub enum EventStatus {
    Planning,
    Registration,
    Scheduled,
    Ongoing,
    Ended,
}

#[table(name = event_template,public)]
pub struct EventTemplate {
    #[auto_inc]
    #[primary_key]
    id: u128,

    name: String,
}

/// Adds a new Event to the specified Tournament.
#[reducer]
pub fn add_event(ctx: &ReducerContext, name: String, to: u128, with: Option<u128>) {
    //TODO authorization
    if let Some(mut tournamet) = ctx.db.tournament().id().find(to) {
        let event = ctx.db.tournament_event().insert(TournamentEvent {
            id: 0,
            tournament: to,
            name,
            status: EventStatus::Planning,
            stages: Vec::new(),
        });

        tournamet.add_event(event.id);

        ctx.db.tournament().id().update(tournamet);
    }
}

#[reducer]
pub fn create_event_template(ctx: &ReducerContext, name: String /* config:  */) {}
