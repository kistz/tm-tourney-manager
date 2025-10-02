use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};
use tm_server_types::method::Method;

#[table(name=tm_server, public)]
pub struct TmServer {
    /// Trackmania provisiones a unique server_id for each server.
    //#[unique]
    #[primary_key]
    pub id: String,

    /// Each server also has a ubisoft account associated with it.
    owner_id: String,

    online: bool,

    active_match: Option<u64>,

    // TODO: Properly enfoce the protocol.
    /// On every update call this MUST be set to None EXCEPT you want to call a method.
    server_method: Option<Method>,
}

/* #[derive(Debug, SpacetimeType)]
pub enum ServerState {
    Available,
    Provisioned,
} */

impl TmServer {
    pub fn active_mactch(&self) -> Option<u64> {
        self.active_match
    }

    pub fn set_active_match(&mut self, match_id: u64) {
        if self.active_match.is_none() {
            self.active_match = Some(match_id)
        }
    }

    /* pub fn set_command(&mut self, command: Method) {
        self.server_method = command
    } */
}

#[cfg(feature = "development")]
#[reducer]
pub fn add_server(ctx: &ReducerContext, id: String) {
    ctx.db.tm_server().insert(TmServer {
        online: true,
        id,
        active_match: None,
        owner_id: "test_user".into(),
        server_method: None,
    });
}

#[cfg(feature = "development")]
#[reducer]
pub fn call_server(ctx: &ReducerContext, id: String, method: Method) {
    if let Some(server) = ctx.db.tm_server().id().find(id) {
        ctx.db.tm_server().id().update(TmServer {
            server_method: Some(method),
            ..server
        });
    }
}
