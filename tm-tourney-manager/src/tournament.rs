use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};

use crate::leaderboard::Leaderboard;

#[table(name = tournament,public)]
pub struct Tournament {
    #[auto_inc]
    #[primary_key]
    id: u128,

    creator: String,
    owners: Vec<String>,

    name: String,

    status: TournamentStatus,

    events: Vec<u128>,

    leaderboard: Option<Leaderboard>,
}

#[derive(Debug, SpacetimeType)]
pub enum TournamentStatus {
    Planning,
    Announced,
    Ongoing,
    Ended,
}

#[reducer]
fn create_tournament(ctx: &ReducerContext, name: String) {
    ctx.db.tournament().insert(Tournament {
        //Extracted from request
        name,
        creator: "yomama".into(),

        //Default values inserted on creation
        id: 0,
        status: TournamentStatus::Planning,
        owners: Vec::new(),
        events: Vec::new(),
        leaderboard: None,
    });
}
