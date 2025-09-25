use spacetimedb::table;
use tm_server_types::event::Event;

// The table name needs to be plural since match is a rust keyword
#[table(name = matches, public)]
pub struct Match {
    #[auto_inc]
    #[primary_key]
    match_id: u128,
}

#[table(name = match_events,public)]
pub struct MatchEvents {
    #[auto_inc]
    #[primary_key]
    id: u128,

    match_id: String,

    event: Event,
}
