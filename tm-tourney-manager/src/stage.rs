use spacetimedb::table;

#[table(name = stage,public)]
pub struct Stage {
    #[auto_inc]
    #[primary_key]
    stage_id: u128,

    matches: Vec<u128>,
}
