use spacetimedb::{SpacetimeType, table};
use tm_server_types::event::Event;

// The table name needs to be plural since match is a rust keyword
#[table(name = matches, public)]
pub struct Match {
    #[auto_inc]
    #[primary_key]
    match_id: u128,

    template: u128,

    status: MatchStatus,
    //server: Entit
}

#[derive(Debug, SpacetimeType)]
pub enum MatchStatus {
    Upcoming,
    Live,
    Ended,
}

#[table(name = server_event,public)]
pub struct ServerEvent {
    #[auto_inc]
    #[primary_key]
    id: u128,

    match_id: String,

    event: Event,
}

#[table(name = match_template,public)]
pub struct MatchTemplate {
    #[auto_inc]
    #[primary_key]
    id: u128,

    creator: String,
}
