use spacetimedb::table;

#[table(name = tournament_stage,public)]
pub struct TournamentStage {
    #[auto_inc]
    #[primary_key]
    stage_id: u128,

    matches: Vec<u128>,
}

#[table(name = stage_template,public)]
pub struct StageTemplate {}
