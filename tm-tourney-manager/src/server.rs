use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};

#[table(name=tm_server, public)]
pub struct TmServer {
    /// Trackmania provisiones a unique server_id for each server.
    #[unique]
    #[primary_key]
    server_id: String,

    /// Each server also has a ubisoft account associated with it.
    owner_id: String,

    online: bool,

    server_command: ServerCommand,
}

#[derive(Debug, SpacetimeType)]
pub struct ServerCommand {
    pause: bool,
}

#[cfg(feature = "development")]
#[reducer]
pub fn add_server(ctx: &ReducerContext, id: String) {
    ctx.db.tm_server().insert(TmServer {
        online: true,
        server_id: id,
        owner_id: "test_user".into(),
        server_command: ServerCommand { pause: false },
    });
}

#[cfg(feature = "development")]
#[reducer]
pub fn call_server(ctx: &ReducerContext, id: String) {
    log::debug!("updated");
    if let Some(server) = ctx.db.tm_server().server_id().find(id) {
        ctx.db.tm_server().server_id().update(TmServer {
            server_command: ServerCommand {
                pause: !server.server_command.pause,
            },
            ..server
        });
    }
}
