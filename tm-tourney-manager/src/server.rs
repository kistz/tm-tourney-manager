use spacetimedb::{SpacetimeType, table};

#[table(name=server, public)]
pub struct Server {
    /* #[auto_inc]
    #[primary_key]
    id: u128, */
    online: bool,

    /// Trackmania provisiones a unique server_id for each server.
    #[unique]
    #[primary_key]
    server_id: String,

    /// Each server also has a ubisoft account associated with it.
    owner_id: String,

    server_command: ServerCommand,
}

#[derive(Debug, SpacetimeType)]
pub struct ServerCommand {
    pause: bool,
}
