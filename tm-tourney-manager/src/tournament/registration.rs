use spacetimedb::{SpacetimeType, Timestamp};

#[derive(Debug, SpacetimeType)]
pub enum Registration {
    Players(PlayerRegistration),
    Team(TeamRegistration),
}

#[derive(Debug, SpacetimeType)]
pub struct PlayerRegistration {
    player_limit: Option<u32>,
    players: Vec<String>,
}

#[derive(Debug, SpacetimeType)]
pub struct TeamRegistration {
    team_limit: Option<u32>,
    team_size_min: u8,
    team_size_max: u8,
    teams: Vec<Team>,
}

#[derive(Debug, SpacetimeType)]
pub struct Team {
    registered_at: Timestamp,
    name: String,
    members: Vec<String>,
}
