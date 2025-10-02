use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};
use tm_server_types::event::Event;

use crate::{
    leaderboard::Leaderboard,
    server::{TmServer, tm_server},
    stage::event_stage,
};

// The table name needs to be plural since match is a rust keyword
#[table(name = stage_match, public)]
pub struct StageMatch {
    #[auto_inc]
    #[primary_key]
    id: u128,

    stage_id: u128,

    server_id: Option<String>,

    //template: u128,
    status: MatchStatus,
    //leaderboard: Leaderboard,
}

#[derive(Debug, SpacetimeType)]
pub enum MatchStatus {
    Configuring,
    Upcoming,
    Live,
    Ended,
}

/// Provisions a new StageMatch to the specified EventStage and a MatchTemplate.
#[reducer]
pub fn provision_match(
    ctx: &ReducerContext,
    to: u128,
    with_config: Option<u128>,
    auto_provisioning_server: bool,
) {
    //TODO authorization
    if let Some(mut stage) = ctx.db.event_stage().id().find(to) {
        let stage_match = ctx.db.stage_match().insert(StageMatch {
            id: 0,
            stage_id: to,
            status: MatchStatus::Upcoming,
            server_id: if auto_provisioning_server {
                //TODO: make auto provisioning logic
                None
            } else {
                None
            },
        });
        stage.add_match(stage_match.id);

        ctx.db.event_stage().id().update(stage);
    }
}

/// Assigns a server to the selected match.
#[reducer]
pub fn assign_server(ctx: &ReducerContext, to: u128, server_id: String) {
    //TODO authorization
    if let Some(mut server) = ctx.db.tm_server().id().find(&server_id)
        && server.active_mactch().is_none()
        && let Some(stage_match) = ctx.db.stage_match().id().find(to)
    {
        let stage_match = ctx.db.stage_match().id().update(StageMatch {
            server_id: Some(server_id),
            ..stage_match
        });

        server.set_active_match(stage_match.id);

        ctx.db.tm_server().id().update(server);
    }
}

/// If the match is fully configured and ready start.
#[reducer]
pub fn try_start(ctx: &ReducerContext, match_id: u128) {
    //TODO authorization
    if let Some(stage_match) = ctx.db.stage_match().id().find(match_id)
        && let Some(server) = stage_match.server_id
        && let Some(mut server) = ctx.db.tm_server().id().find(server)
    {
        ctx.db.tm_server().id().update(server);
    }
}

#[table(name = match_template,public)]
pub struct MatchTemplate {
    #[auto_inc]
    #[primary_key]
    id: u128,

    creator: String,
}

#[table(name = tm_server_event,public)]
pub struct TmServerEvent {
    #[auto_inc]
    #[primary_key]
    id: u128,

    match_id: u128,

    event: Event,
}

// TODO: remove the id argument and get it from calling entity.
#[reducer]
pub fn post_event(ctx: &ReducerContext, id: String, event: Event) {
    if let Some(server) = ctx.db.tm_server().id().find(id)
        && let Some(match_id) = server.active_mactch()
    {
        ctx.db.tm_server_event().insert(TmServerEvent {
            id: 0,
            match_id,
            event,
        });
    }
}
