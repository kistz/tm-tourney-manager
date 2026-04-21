use spacetimedb::{Table, table};
use tm_server_types::config::ServerConfig;

use crate::{
    competition::node::NodeHandle, raw_server::occupation::TabRawServerOccupationRead,
    tm_match::tab_match__view, tm_server::tab_server__view,
};

#[table(accessor=tab_raw_server_config)]
struct RawServerConfig {
    #[auto_inc]
    #[primary_key]
    id: u32,

    config: ServerConfig,
}

impl RawServerConfig {
    pub fn new(config: ServerConfig) -> Self {
        Self { id: 0, config }
    }
}

#[table(accessor=event_raw_server_state,event,public)]
struct EventRawServerState {
    #[primary_key]
    server_id: u32,
    config: ServerConfig,
    open: bool,
    skip_again: bool,
    seamless: bool,
}

pub(crate) trait RawServerContigRead {
    fn raw_server_config_references(&self, config_id: u32) -> Vec<NodeHandle>;
}
impl<Db: spacetimedb::DbContext> RawServerContigRead for Db {
    fn raw_server_config_references(&self, config_id: u32) -> Vec<NodeHandle> {
        let mut config_references = Vec::new();
        config_references.extend(
            self.db_read_only()
                .tab_match()
                .config()
                .filter(config_id)
                .map(|m| NodeHandle::MatchV1(m.id)),
        );
        config_references.extend(
            self.db_read_only()
                .tab_match()
                .pre_config()
                .filter(config_id)
                .map(|m| NodeHandle::MatchV1(m.id)),
        );
        config_references.extend(
            self.db_read_only()
                .tab_server()
                .config()
                .filter(config_id)
                .map(|m| NodeHandle::ServerV1(m.id)),
        );

        config_references
    }
}

pub(crate) trait RawServerContigWrite {
    fn raw_server_config_update(
        &self,
        config_id: u32,
        new_config: ServerConfig,
    ) -> Result<(), String>;

    fn raw_server_config_new(&self, new_config: ServerConfig) -> Result<u32, String>;

    fn emit_raw_server_config(&self, server_id: u32, seamless: bool) -> Result<(), String>;
}

impl<Db: spacetimedb::DbContext<DbView = spacetimedb::Local>> RawServerContigWrite for Db {
    fn raw_server_config_update(
        &self,
        config_id: u32,
        new_config: ServerConfig,
    ) -> Result<(), String> {
        self.db()
            .tab_raw_server_config()
            .id()
            .update(RawServerConfig {
                id: config_id,
                config: new_config,
            });

        Ok(())
    }

    fn raw_server_config_new(&self, new_config: ServerConfig) -> Result<u32, String> {
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
                        skip_again: tm_match.is_recovery(),
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
                        skip_again: false,
                        seamless,
                    })?;

                Ok(())
            }
            _ => {
                log::error!(
                    "Requested a configuration from a node type other than Match or Server?"
                );
                Err("Requested a configuration from a node type other than Match or Server?".into())
            }
        }
    }
}

// How could we archieve adaptive configuration that changes dynamically with players???
// How could we filter the players better?

// -> Yoink 16 -> Yoink 16.
// How would we implement a conditional node????
