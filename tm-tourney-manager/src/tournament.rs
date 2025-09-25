use spacetimedb::table;

#[table(name = tournament,public)]
pub struct Tournament {
    #[auto_inc]
    #[primary_key]
    id: u128,

    creator: String,

    events: Vec<u128>,
}
