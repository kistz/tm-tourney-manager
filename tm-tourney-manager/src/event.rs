use spacetimedb::table;

#[table(name = tournament_event,public)]
pub struct TournamentEvent {
    #[auto_inc]
    #[primary_key]
    event_id: u128,

    //Scheduled time
    starting: String,

    template: u128,

    stages: Vec<u128>,
}
