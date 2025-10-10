use spacetimedb::{ReducerContext, SpacetimeType, Table, reducer, table};
use tm_server_types::{config::ServerConfig, event::Event};

use crate::{
    leaderboard::Leaderboard,
    server::{TmServer, tm_server},
    stage::{self, event_stage},
};

// The table name needs to be plural since match is a rust keyword
#[table(name = stage_match, public)]
pub struct StageMatch {
    #[auto_inc]
    #[primary_key]
    pub id: u64,

    /// The stage this match is associated with.
    stage_id: u64,
    /// The assigned server that will be used by this match.
    server_id: Option<String>,

    /// The moment the server is assigned to the match the pre_match_config gets loaded in.
    /// Only if it is defined. Useful for hiding tournament maps till the actual start.
    pre_match_config: Option<ServerConfig>,
    /// If the match is started this config gets loaded.
    /// Has to be specified before your able to advance into Upcoming.
    match_config: Option<ServerConfig>,
    post_match_config: Option<ServerConfig>,

    status: MatchStatus,
    //leaderboard: Leaderboard,
}

impl StageMatch {
    /// Evaluates is the Match is in the "Match" state of its lifecycle.
    pub fn is_match(&self) -> bool {
        self.status == MatchStatus::Match
    }
}

#[derive(Debug, SpacetimeType, PartialEq, Eq)]
pub enum MatchStatus {
    Configuring,
    Upcoming,
    PreMatch,
    Match,
    PostMatch,
    Ended,
}

/// Provisions a new StageMatch to the specified EventStage and a MatchTemplate.
#[reducer]
pub fn provision_match(
    ctx: &ReducerContext,
    used_by: u64,
    with_config: Option<u64>,
    auto_provisioning_server: bool,
) {
    //TODO authorization
    if let Some(mut stage) = ctx.db.event_stage().id().find(used_by) {
        let stage_match = ctx.db.stage_match().insert(StageMatch {
            id: 0,
            stage_id: used_by,
            status: MatchStatus::Configuring,
            server_id: if auto_provisioning_server { None } else { None },
            pre_match_config: None,
            match_config: None,
            post_match_config: None,
        });
        stage.add_match(stage_match.id);

        ctx.db.event_stage().id().update(stage);
    }
}

/// Assigns a server to the selected match.
#[reducer]
pub fn match_assign_server(ctx: &ReducerContext, to: u64, server_id: String) {
    //TODO authorization
    if let Some(mut server) = ctx.db.tm_server().id().find(&server_id)
        && server.active_mactch().is_none()
        && let Some(stage_match) = ctx.db.stage_match().id().find(to)
        && stage_match.status == MatchStatus::Configuring
    {
        let stage_match = ctx.db.stage_match().id().update(StageMatch {
            server_id: Some(server_id),
            ..stage_match
        });

        server.set_active_match(stage_match.id);

        ctx.db.tm_server().id().update(server);
    }
}

#[reducer]
pub fn match_configured(ctx: &ReducerContext, id: u64) {
    //TODO authorization
    if let Some(mut stage_match) = ctx.db.stage_match().id().find(id)
        && stage_match.status == MatchStatus::Configuring
    {
        stage_match.status = MatchStatus::Upcoming;
        ctx.db.stage_match().id().update(stage_match);
    }
}

/* #[reducer]
pub fn update_pre_match_config(ctx: &ReducerContext, id: u64, config: ServerConfig) {
    //TODO authorization
    if let Some(mut stage_match) = ctx.db.stage_match().id().find(id) {
        stage_match.match_config = Some(config);
        ctx.db.stage_match().id().update(stage_match);
    }
} */

#[reducer]
pub fn update_match_config(ctx: &ReducerContext, id: u64, config: ServerConfig) {
    //TODO authorization
    if let Some(mut stage_match) = ctx.db.stage_match().id().find(id) {
        stage_match.match_config = Some(config);
        ctx.db.stage_match().id().update(stage_match);
    }
}

/// If the match is fully configured and ready start.
#[reducer]
pub fn try_start(ctx: &ReducerContext, match_id: u64) {
    //TODO authorization
    if let Some(stage_match) = ctx.db.stage_match().id().find(match_id)
        && let Some(server) = stage_match.server_id
        && let Some(mut server) = ctx.db.tm_server().id().find(server)
        && let Some(config) = stage_match.match_config
        && stage_match.status == MatchStatus::Upcoming
    {
        server.set_config(config);

        ctx.db.tm_server().id().update(server);
    }
}

#[table(name = match_template,public)]
pub struct MatchTemplate {
    #[auto_inc]
    #[primary_key]
    id: u64,

    creator: String,
}
