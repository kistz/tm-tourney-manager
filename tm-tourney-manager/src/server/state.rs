use spacetimedb::SpacetimeType;

/// Makinig the server completly stateless and only a shell for physics calculation and managing the players.
#[derive(Debug, SpacetimeType)]
pub struct ServerState {
    players: Vec<String>,
    paused: bool,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            players: Default::default(),
            paused: false,
        }
    }
}
