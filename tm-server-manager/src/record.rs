use spacetimedb::{SpacetimeType, Timestamp, Uuid};

//mod competition;
//mod map;
//mod r#match;

/// General purpose Record Type returned by query all sorts of leaderboards in the project.
/// All entries to a leaderboard should have a replay or ghost associated with it.
#[derive(Debug /* SpacetimeType */)]
pub struct TmRecord {
    pub map_id: u32,
    pub user_id: u32,

    pub timestamp: Timestamp,
    pub time: u32,
}
