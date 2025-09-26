use spacetimedb::{SpacetimeType, table};

#[derive(Debug, SpacetimeType)]
pub struct Leaderboard {
    players: Vec<String>,
}
