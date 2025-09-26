use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};

use crate::leaderboard::Leaderboard;

#[table(name = tournament_event,public)]
pub struct TournamentEvent {
    #[auto_inc]
    #[primary_key]
    event_id: u128,

    /// The template used for this Event.
    template: u128,
    /// To which instantiation the template belongs.
    tournament: u128,

    status: EventStatus,

    //Scheduled time
    starting: String,

    stages: Vec<u128>,

    leaderboard: Leaderboard,
}

#[derive(Debug, SpacetimeType)]
pub enum EventStatus {
    Registration,
    Scheduled,
    Ongoing,
    Ended,
}

#[table(name = event_template,public)]
pub struct EventTemplate {
    name: String,
}

#[reducer]
pub fn add_event(ctx: &ReducerContext, with: u128, to: u128) {
    //TODO
    /* ctx.db.event().insert(Event {
        event_id: 0,
        template: with,
        tournament: to,
        status: todo!(),
        starting: todo!(),
        stages: todo!(),
        leaderboard: todo!(),
    }); */
}
