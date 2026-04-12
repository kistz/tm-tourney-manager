use spacetimedb::{SpacetimeType, Table, ViewContext, table, view};
use tm_server_types::config::ServerConfig;

use crate::{
    competition::node::NodeRead,
    raw_server::{occupation::TabRawServerOccupationRead, tab_raw_server__view},
    tm_match::{MatchStatus, tab_match__view},
    tm_server::tab_server__view,
};

#[table(accessor=tab_raw_server_config)]
struct RawServerConfig {
    #[auto_inc]
    #[primary_key]
    id: u32,

    config: ServerConfig,
}

impl RawServerConfig {
    /* pub fn get_config(self) -> ServerConfig {
        self.config
    } */

    pub fn new(config: ServerConfig) -> Self {
        Self { id: 0, config }
    }
}

// The configuration that is owned by a server.
/* #[table(accessor=tab_raw_server_config_active)]
pub struct RawServerConfigActive {
    config: u32,

    #[primary_key]
    pub server_login: String,
}

impl RawServerConfigActive {
    /// Returns a new defualt config
    pub(crate) fn new(server_login: String) -> Self {
        Self {
            //TODO
            config: 0,
            server_login,
        }
    }
} */

/* #[spacetimedb::reducer]
pub fn create_server_config(ctx: &ReducerContext, config: ServerConfig) -> Result<(), String> {
    let user = ctx.get_user()?;

    ctx.db.tab_raw_server_config().try_insert(RawServerConfig {
        id: 0,
        //account_id: user.account_id,
        config,
    })?;

    Ok(())
} */

/* #[derive(Debug, SpacetimeType)]
struct ServerMetadata {
    config: ServerConfig,
    open: bool,
    force_restart: bool,
} */

/* #[view(accessor=raw_server_config,public)]
fn raw_server_config(ctx: &ViewContext) -> Option<ServerMetadata> {
    let server = ctx.db.tab_raw_server().identity().find(ctx.sender())?;

    let node = ctx.raw_server_occupation(server.id)?;

    match node {
        crate::competition::node::NodeHandle::MatchV1(m) => {
            let tm_match = ctx.db.tab_match().id().find(m).unwrap();
            let config = ctx
                .db
                .tab_raw_server_config()
                .id()
                .find(tm_match.get_config_id())?;
            Some(ServerMetadata {
                config: config.config,
                open: tm_match.is_open(),
                force_restart: tm_match.force_restart(),
            })
        }
        crate::competition::node::NodeHandle::ServerV1(s) => {
            let tm_server = ctx.db.tab_server().id().find(s).unwrap();
            let config = ctx
                .db
                .tab_raw_server_config()
                .id()
                .find(tm_server.get_config_id())?;
            Some(ServerMetadata {
                config: config.config,
                open: tm_server.is_open(),
                //TODO,
                force_restart: false,
            })
        }
        _ => {
            log::error!("Requested a configuration from a node type other than Match or Server?");
            None
        }
    }
} */

#[table(accessor=event_raw_server_state,event,public)]
struct EventRawServerState {
    #[primary_key]
    server_id: u32,
    config: ServerConfig,
    open: bool,
    recovery_section: bool,
    seamless: bool,
}

pub(crate) trait RawServerContigWrite {
    fn raw_server_config_update(
        &self,
        server_id: u32,
        new_config: ServerConfig,
    ) -> Result<u32, String>;

    fn raw_server_match_config_override(
        &self,
        match_id: u32,
        new_config: ServerConfig,
    ) -> Result<u32, String>;

    fn emit_raw_server_config(&self, server_id: u32, seamless: bool) -> Result<(), String>;
}

impl<Db: spacetimedb::DbContext<DbView = spacetimedb::Local>> RawServerContigWrite for Db {
    fn raw_server_config_update(
        &self,
        server_id: u32,
        new_config: ServerConfig,
    ) -> Result<u32, String> {
        todo!()
    }

    fn raw_server_match_config_override(
        &self,
        match_id: u32,
        new_config: ServerConfig,
    ) -> Result<u32, String> {
        //TODO clean up old config or smth.

        let id = self
            .db()
            .tab_raw_server_config()
            .try_insert(RawServerConfig::new(new_config))?;

        Ok(id.id)
    }

    fn emit_raw_server_config(&self, server_id: u32, seamless: bool) -> Result<(), String> {
        let Some(node) = self.raw_server_occupation(server_id) else {
            return Err("Ocuupation not found.".into());
        };

        match node {
            crate::competition::node::NodeHandle::MatchV1(m) => {
                let tm_match = self.db_read_only().tab_match().id().find(m).unwrap();
                let Some(config) = self
                    .db_read_only()
                    .tab_raw_server_config()
                    .id()
                    .find(tm_match.get_config_id())
                else {
                    return Err("Cannot find config.".into());
                };
                self.db()
                    .event_raw_server_state()
                    .try_insert(EventRawServerState {
                        server_id,
                        config: config.config,
                        open: tm_match.is_open(),
                        recovery_section: tm_match.is_recovery(),
                        seamless,
                    })?;

                Ok(())
            }
            crate::competition::node::NodeHandle::ServerV1(s) => {
                let tm_server = self.db_read_only().tab_server().id().find(s).unwrap();
                let config = self
                    .db_read_only()
                    .tab_raw_server_config()
                    .id()
                    .find(tm_server.get_config_id())
                    .unwrap();
                self.db()
                    .event_raw_server_state()
                    .try_insert(EventRawServerState {
                        server_id,
                        config: config.config,
                        open: tm_server.is_open(),
                        recovery_section: false,
                        seamless,
                    })?;

                Ok(())
            }
            _ => {
                log::error!(
                    "Requested a configuration from a node type other than Match or Server?"
                );
                Err(
                    "Requested a configuration from a node type other than Match or Server?".into(),
                )
            }
        }
    }
}
