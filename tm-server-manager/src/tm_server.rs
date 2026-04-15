use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};
use tm_server_types::config::ServerConfig;

use crate::{
    authorization::Authorization,
    competition::{
        CompetitionPermissionsV1,
        node::{NodeHandle, NodeWrite},
        server_pool::TabCompetitionServerPoolRead,
        tab_competition,
    },
    raw_server::{
        TabRawServerWrite,
        config::RawServerContigWrite,
        occupation::{TabRawServerOccupationRead, TabRawServerOccupationWrite},
        tab_raw_server,
    },
    tm_server::template::server_template_instantiate,
};

pub mod template;

#[table(accessor= tab_server)]
pub struct ServerV1 {
    name: String,

    #[auto_inc]
    #[primary_key]
    pub(crate) id: u32,

    #[index(hash)]
    parent_id: u32,

    config: u32,

    status: ServerStatus,

    open: bool,
}

impl ServerV1 {
    pub(crate) fn get_config_id(&self) -> u32 {
        self.config
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }
}

#[derive(Debug, PartialEq, Eq, SpacetimeType, Clone, Copy)]
pub enum ServerStatus {
    Configuring,
    Ongoing,
}

#[reducer]
pub fn server_create(
    ctx: &ReducerContext,
    name: String,
    parent_id: u32,
    with_template: u32,
) -> Result<(), String> {
    let Some(parent_competition) = ctx.db.tab_competition().id().find(parent_id) else {
        return Err("Invalid competition".into());
    };

    ctx.auth_builder(parent_id)
        .permission(CompetitionPermissionsV1::SERVER_CREATE)
        .authorize()?;

    if parent_competition.is_template() {
        return Err(
            "Cannot add a normal server to a template. Try do add a template server to id.".into(),
        );
    }

    // Try to load template if provided
    if with_template != 0 {
        server_template_instantiate(ctx, with_template)?;
    } else {
        // Create an uncommitted match
        let tm_server = ServerV1 {
            name,
            id: 0,
            parent_id,
            config: 0,
            status: ServerStatus::Configuring,
            open: true,
        };

        let tm_server = ctx.db.tab_server().try_insert(tm_server)?;

        ctx.node_create(NodeHandle::ServerV1(tm_server.id))?;
    }

    Ok(())
}

// Select 0 for auto provisioning.
#[reducer]
pub fn server_assign_raw_server(
    ctx: &ReducerContext,
    to: u32,
    server_id: u32,
) -> Result<(), String> {
    let Some(tm_server) = ctx.db.tab_server().id().find(to) else {
        return Err("Supplied match was not found!".into());
    };

    ctx.auth_builder(tm_server.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_ASSIGN_SERVER)
        .authorize()?;

    //auto provision
    if server_id == 0 {
        ctx.raw_server_pool_assign(NodeHandle::ServerV1(tm_server.id))?;
    } else {
        if ctx.raw_server_is_occupied(server_id) {
            return Err("Server is already occupied! Cannot assign!".into());
        }

        if ctx.db.tab_raw_server().id().find(server_id).is_none() {
            return Err("Server with id was not found!".into());
        };

        if ctx
            .server_pool_available(tm_server.parent_id)
            .into_iter()
            .any(|s| s.id == server_id)
        {
            return Err("Server is not lended to the project".into());
        }

        ctx.raw_server_occupation_add(NodeHandle::ServerV1(to), server_id)?;
    }

    Ok(())
}

#[reducer]
pub fn server_configured(ctx: &ReducerContext, id: u32) -> Result<(), String> {
    let Some(mut tm_server) = ctx.db.tab_server().id().find(id) else {
        return Err("Server was mot found!".into());
    };

    ctx.auth_builder(tm_server.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_CONFIGURE)
        .authorize()?;

    if tm_server.status != ServerStatus::Configuring {
        return Err("Match is not in configuring state".into());
    }

    if tm_server.config == 0 {
        return Err("Need a configuration in order to set server to configured.".into());
    }

    tm_server.status = ServerStatus::Ongoing;

    ctx.db.tab_server().id().update(tm_server);

    Ok(())
}

#[reducer]
pub fn server_config_override(
    ctx: &ReducerContext,
    to: u32,
    config: ServerConfig,
) -> Result<(), String> {
    let Some(mut tm_server) = ctx.db.tab_server().id().find(to) else {
        return Err("Supplied match was not found!".into());
    };

    ctx.auth_builder(tm_server.parent_id)
        //TODO
        //.permission(CompetitionPermissionsV1::SERVER_ASSIGN)
        .authorize()?;

    /* ctx.raw_server_config_update(server_id, new_config)

    tm_server.config = config */

    //TODO
    Ok(())
}

// Server and match config problem space mapping:
// How to clean up old configs which are orphaned or override them?
// Big problem: Are there crossovers between servers and matches?
//  -> This invariant should be impossible??? -> otherwise how to handle
//  -> Prefferably handle that case.
//

// -> Differentiate between update and override.
// -> Annotate a hash index for match and server configs.
// -> Always search all three things: server config. match config. match pre_config.
// -> when no other stuff is found then we can delete it or even update?
// -> override is conceptually obvious i guess -> we can update the old one even if override if its the only one.

// -> what should the confirmation be for updating a random config?
// -> maybe expose a procedure which warns you with the count of how many configs you are updating?

// Problem: what if there are matches in progress for a config update?
// -> does the update just fail?
// -> 
