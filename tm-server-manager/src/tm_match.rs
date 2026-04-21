use std::time::Duration;

use spacetimedb::{
    Query, ReducerContext, SpacetimeType, Table, TimeDuration, ViewContext, reducer, table, view,
};
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
        TabRawServerRead, TabRawServerWrite,
        config::RawServerContigWrite,
        destination::TabRawServerDestinationWrite,
        occupation::{TabRawServerOccupationRead, TabRawServerOccupationWrite},
        tab_raw_server,
    },
    tm_match::{
        auto_recovery::RecoveryWrite,
        leaderboard::{tab_match_round_player, tab_match_round_player_ext},
        state::{MatchState, tab_match_state},
        template::match_template_instantiate,
    },
};

mod auto_recovery;
pub mod event;
pub mod leaderboard;
pub mod replay;
pub mod state;
pub mod template;

/// # Match
/// Fullfills the role of providing configuration to the associated server and
/// executes the match on a Trackmania Server.
/// Also holds the Rules to reconstruct a Leaderboard for the match.
///
/// ## Lifecycle
/// Is represented and can be queried via the [MatchStatus]
/// and consists of:
/// - *Created.* In order to advance to the next stage a valid configuration for
///  match_config need to be present. The same config will be used for pre_match if not overridden.
///  Advances to [MatchStatus::Configuring].
/// - *Configured.* Advances to [MatchStatus::Upcoming].
/// - *Captured Server.* Capturing describes the process of assigning a
/// Server from the pool to the Match. The server is locked till the match
/// releases it again. Advances to [MatchStatus::PreMatch]
/// - *Start.* Can be called manually, with a schedule or with rules.  
/// If the ephemeral state matches the desired state. Advances to [MatchStatus::Live].
/// - *End.* The match has concluded. Loads the post_match_config if it is present. Releases
/// the captured server. Advances to [MatchStatus::Ended].
#[table(accessor= tab_match)]
pub struct MatchV1 {
    name: String,

    #[auto_inc]
    #[primary_key]
    pub(crate) id: u32,

    #[index(hash)]
    parent_id: u32,

    /// The moment the server is captured by the match the pre_match_config gets loaded in.
    /// Only if it is defined. Useful for hiding project maps till the actual start.
    pre_config: u32,
    /// If the match is started this config gets loaded.
    /// Has to be specified before your able to advance into Upcoming.
    config: u32,

    status: MatchStatus,

    auto_provision_server: bool,
    //Whether the match is open for all to join or restricted.
    open: bool,
    template: bool,
    // Used for force restart. If status changes this should get set to true and false again
    // to trigger a config refresh on the raw_server.
    // TODOMaybe just send an event mhm.
    //dirty: bool,
}

impl MatchV1 {
    pub fn get_config_id(&self) -> u32 {
        match self.status {
            MatchStatus::Configuring => {
                panic!("should not ask for a config if match is configuring.")
            }
            MatchStatus::Configured => {
                panic!("should not ask for a config if match is configured.")
            }
            MatchStatus::Preparation => {
                if self.pre_config != 0 {
                    self.pre_config
                } else {
                    self.config
                }
            }
            MatchStatus::Live => self.config,
            MatchStatus::Ended => self.config,
            MatchStatus::Locked => {
                panic!("should not ask for a config if match is locked.")
            }
            // This is for the seamless discovery so it is treated as a live match.
            MatchStatus::Recovery => self.config,
            // This should get conceptually treated as preparing.
            MatchStatus::RecoveryPreparation => {
                if self.pre_config != 0 {
                    self.pre_config
                } else {
                    self.config
                }
            }
        }
    }

    /// Evaluates is the Match is in the "Match" state of its lifecycle.
    pub fn is_live(&self) -> bool {
        self.status == MatchStatus::Live
    }

    pub fn is_recovery(&self) -> bool {
        self.status == MatchStatus::Recovery
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn status(&self) -> MatchStatus {
        self.status
    }

    pub fn get_comp_id(&self) -> u32 {
        self.parent_id
    }

    pub fn is_template(&self) -> bool {
        self.template
    }

    pub(crate) fn instantiate(mut self, parent_id: u32, stay_template: bool) -> Self {
        self.template = stay_template;
        self.parent_id = parent_id;
        self.id = 0;
        self
    }

    pub(crate) fn end_match(&mut self) {
        self.status = MatchStatus::Ended;
    }

    pub(crate) fn enter_recovery(&mut self) {
        self.status = MatchStatus::Recovery;
    }

    /* pub(crate) fn force_restart(&self) -> bool {
        self.dirty
    } */
}

#[derive(Debug, PartialEq, Eq, SpacetimeType, Clone, Copy)]
pub enum MatchStatus {
    /// Allows to change all associated configurations of the Match.
    Configuring,
    Configured,
    /// No changes to the pre_match configuration can be made anymore.
    Preparation,
    /// No changes to the match configuration can be made anymore.
    Live,
    /// Match is immutable and achived.
    /// Loads the post match config if present.
    Ended,
    Locked,
    Recovery,
    RecoveryPreparation,
}

impl MatchStatus {
    fn before_preparation(&self) -> bool {
        match self {
            MatchStatus::Configuring => true,
            MatchStatus::Configured => true,
            MatchStatus::Preparation => false,
            MatchStatus::Live => false,
            MatchStatus::Ended => false,
            MatchStatus::Locked => false,
            MatchStatus::Recovery => false,
            MatchStatus::RecoveryPreparation => false,
        }
    }
}

#[reducer]
pub fn match_create(
    ctx: &ReducerContext,
    name: String,
    parent_id: u32,
    with_template: u32,
) -> Result<(), String> {
    let Some(parent_competition) = ctx.db.tab_competition().id().find(parent_id) else {
        return Err("Invalid competition".into());
    };

    ctx.auth_builder(parent_id)
        .permission(CompetitionPermissionsV1::MATCH_CREATE)
        .authorize()?;

    if parent_competition.is_template() {
        return Err(
            "Cannot add a normal match to a template. Try do add a template match to id.".into(),
        );
    }

    // Try to load template if provided
    if with_template != 0 {
        match_template_instantiate(ctx, with_template)?;
    } else {
        // Create an uncommitted match
        let tm_match = MatchV1 {
            id: 0,
            parent_id,
            name,
            status: MatchStatus::Configuring,
            pre_config: 0,
            config: 0,
            auto_provision_server: true,
            template: false,
            open: false,
            //dirty: false,
        };

        let tm_match = ctx.db.tab_match().try_insert(tm_match)?;
        ctx.node_create(NodeHandle::MatchV1(tm_match.id))?;
    }

    Ok(())
}

/// Assigns a server to the selected match.
/// This is only possible if the match is configuring or down
/// the server is not already occupied
/// the user has the permission to assign servers in the project
/// and the server is lended to the project.
#[reducer]
pub fn match_assign_server(ctx: &ReducerContext, to: u32, server_id: u32) -> Result<(), String> {
    let Some(tm_match) = ctx.db.tab_match().id().find(to) else {
        return Err("Supplied match was not found!".into());
    };

    ctx.auth_builder(tm_match.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_ASSIGN_SERVER)
        .authorize()?;

    if tm_match.status != MatchStatus::Configuring && tm_match.status != MatchStatus::Configured {
        return Err(
            "Match is currently not getting configured so assigning a new server is impossible."
                .into(),
        );
    }

    if ctx.raw_server_is_occupied(server_id) {
        return Err("Server is already occupied! Cannot assign!".into());
    }

    if ctx.db.tab_raw_server().id().find(server_id).is_none() {
        return Err("Server with id was not found!".into());
    };

    if ctx
        .server_pool_available(tm_match.parent_id)
        .into_iter()
        .any(|s| s.id == server_id)
    {
        return Err("Server is not lended to the project".into());
    }

    ctx.raw_server_occupation_add(NodeHandle::MatchV1(to), server_id)?;

    Ok(())
}

#[reducer]
pub fn match_configured(ctx: &ReducerContext, id: u32) -> Result<(), String> {
    let Some(mut tm_match) = ctx.db.tab_match().id().find(id) else {
        return Err("Match was mot found!".into());
    };

    ctx.auth_builder(tm_match.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_CONFIGURE)
        .authorize()?;

    if tm_match.status != MatchStatus::Configuring {
        return Err("Match is not in configuring state".into());
    }
    tm_match.status = MatchStatus::Configured;

    ctx.db.tab_match().id().update(tm_match);

    Ok(())
}

#[reducer]
pub fn match_update_pre_config(
    ctx: &ReducerContext,
    id: u32,
    config_id: u32,
) -> Result<(), String> {
    if let Some(mut tm_match) = ctx.db.tab_match().id().find(id)
        && tm_match.status == MatchStatus::Configuring
    {
        ctx.auth_builder(tm_match.parent_id)
            .permission(CompetitionPermissionsV1::MATCH_CONFIGURE)
            .authorize()?;
        tm_match.pre_config = config_id;
        ctx.db.tab_match().id().update(tm_match);
        Ok(())
    } else {
        Err(format!("Match {id} not found or in wrong state."))
    }
}

#[reducer]
pub fn match_override_config(
    ctx: &ReducerContext,
    id: u32,
    config: ServerConfig,
) -> Result<(), String> {
    let Some(mut tm_match) = ctx.db.tab_match().id().find(id) else {
        return Err("Match was mot found!".into());
    };

    ctx.auth_builder(tm_match.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_CONFIGURE)
        .authorize()?;

    if !tm_match.status.before_preparation() {
        return Err("Too late to set configuration".into());
    }

    //TODO cleanup old/orphaned configs. Should i do this with a mapping table or just always instantiate the config or keep track of this in the match?
    //TODO also check if it is empty (0) or if smth was there before.
    let new_id = ctx.raw_server_match_config_override(tm_match.id, config)?;
    tm_match.config = new_id;
    ctx.db.tab_match().id().update(tm_match);
    Ok(())
}

/// If the match is fully configured and ready start.
/// This can also serve as a manual override for scheduled matches.
#[reducer]
fn match_set_preparation(ctx: &ReducerContext, match_id: u32) -> Result<(), String> {
    let Some(tm_match) = ctx.db.tab_match().id().find(match_id) else {
        return Err("Match not found!".into());
    };

    ctx.auth_builder(tm_match.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_CONFIGURE)
        .authorize()?;

    authorized_match_set_preparation(ctx, match_id)
}

pub fn authorized_match_set_preparation(ctx: &ReducerContext, match_id: u32) -> Result<(), String> {
    let Some(mut tm_match) = ctx.db.tab_match().id().find(match_id) else {
        return Err("Match not found!".into());
    };

    if tm_match.is_template() {
        return Err("Method cannot be called on templates.".into());
    }

    if tm_match.status == MatchStatus::Configuring {
        return Err("Match is still getting configured.".into());
    }
    if tm_match.config == 0 {
        return Err(
            "Match needs a configuration in order to advance to the upcoming state.".into(),
        );
    }

    let server_id = if let Some(server_id) =
        ctx.occupation_with_occupier(NodeHandle::MatchV1(match_id))
    {
        tm_match.status = MatchStatus::Preparation;
        server_id
    } else if tm_match.auto_provision_server {
        let server_id = ctx.raw_server_pool_assign(NodeHandle::MatchV1(match_id))?;

        tm_match.status = MatchStatus::Preparation;
        server_id
    } else {
        return Err("Match has auto provisioning turned off and no server assigned! Cannot start the match!".into());
    };

    ctx.db.tab_match().id().update(tm_match);

    ctx.db
        .tab_match_state()
        .try_insert(MatchState::new(match_id))?;

    ctx.destination_claim(NodeHandle::MatchV1(match_id))?;

    ctx.emit_raw_server_config(server_id, false)?;

    Ok(())
}

/// If the match is fully configured and ready start.
/// This can also serve as a manual override for scheduled matches.
#[reducer]
pub fn match_try_start(ctx: &ReducerContext, match_id: u32) -> Result<(), String> {
    let Some(mut tm_match) = ctx.db.tab_match().id().find(match_id) else {
        return Err("Match not found!".into());
    };

    if tm_match.is_template() {
        return Err("Method cannot be called on templates.".into());
    }

    if tm_match.status != MatchStatus::Preparation {
        return Err("Match needs to be prepared in order to be started.".into());
    }

    ctx.auth_builder(tm_match.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_CONFIGURE)
        .authorize()?;

    let Some(server_id) = ctx.occupation_with_occupier(NodeHandle::MatchV1(match_id)) else {
        return Err("No server is assigned to the match.".into());
    };

    //TODO this is depending on player state (e.g. is there need to be specific players present are all there?)
    tm_match.status = MatchStatus::Live;
    ctx.db.tab_match().id().update(tm_match);

    let mut state = ctx.db.tab_match_state().match_id().find(match_id).unwrap();
    state.set_live();
    ctx.db.tab_match_state().match_id().update(state);

    ctx.emit_raw_server_config(server_id, false)?;

    Ok(())
}

//TODO restore functionality.
/* #[reducer]
pub fn match_delete(ctx: &ReducerContext, match_id: u32) -> Result<(), String> {
    let Some(tm_match) = ctx.db.tab_match().id().find(match_id) else {
        return Err(format!("Match with id: {match_id} not found."));
    };

    ctx.auth_builder(tm_match.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_DELETE)
        .authorize()?;

    if !ctx.db.tab_match().id().delete(match_id) {
        return Err(format!("Match with id: {match_id} not found."));
    }

    let handle = NodeHandle::MatchV1(match_id);

    ctx.node_delete(handle)?;

    Ok(())
} */

#[view(accessor=my_matches,public)]
fn my_matches(ctx: &ViewContext /* competition_id: u32 */) -> impl Query<MatchV1> {
    /* let Ok(user) = ctx.user_id() else {
        log::warn!(
            "Non user account has tried to call protected view: {}",
            ctx.sender()
        );
        return Vec::new();
    }; */

    let competition_id = 1u32;

    //TODO access control for only permitted users. e.g. walk competition tree for permission.

    ctx.from.tab_match()
}

pub(crate) trait MatchRead {}
impl<Db: spacetimedb::DbContext> MatchRead for Db {}

pub(crate) trait MatchWrite: MatchRead {
    fn match_recovery_enter(&self, match_id: u32);
    fn match_recovery_exit_seamless(&self, match_id: u32);
    fn match_recovery_exit_forced(&self, match_id: u32);
    fn match_name_edit(&self, match_id: u32, name: String) -> Result<(), String>;
}
impl<Db: spacetimedb::DbContext<DbView = spacetimedb::Local>> MatchWrite for Db {
    fn match_recovery_enter(&self, match_id: u32) {
        //SAFETY: if a occupation is inserted it must also exist.
        let mut tm_match = self.db().tab_match().id().find(match_id).unwrap();

        if tm_match.is_live() {
            log::error!("MATCH {} ENTERING RECOVERY", tm_match.id);

            tm_match.enter_recovery();
            self.db().tab_match().id().update(tm_match);

            //SAFETY: Match is live so have to have a state.
            let mut state = self
                .db()
                .tab_match_state()
                .match_id()
                .find(match_id)
                .unwrap();

            state.set_pause(true);
            state.set_recovery();

            self.db()
                .tab_match_round_player()
                .match_round()
                .delete((match_id, state.get_round()));
            self.db()
                .tab_match_round_player_ext()
                .match_round()
                .delete((match_id, state.get_round()));

            self.db().tab_match_state().match_id().update(state);

            let raw_server = self
                .occupation_with_occupier(NodeHandle::MatchV1(match_id))
                .unwrap();
            if let Err(err) = self.match_auto_recovery_insert(
                match_id,
                self.raw_server_last_connection(raw_server),
                TimeDuration::from_duration(Duration::from_mins(5)),
            ) {
                log::error!("Could not insert match auto recovery. Reason: {err}")
            };
        }
    }

    fn match_recovery_exit_seamless(&self, match_id: u32) {
        //SAFETY: if a occupation is inserted it must also exist.
        let mut tm_match = self.db().tab_match().id().find(match_id).unwrap();

        if tm_match.is_recovery() {
            log::error!(
                "MATCH {} EXITING RECOVERY SEAMLESSLY BECAUSE SERVER CAME BACK.",
                tm_match.id
            );

            tm_match.status = MatchStatus::Live;
            self.db().tab_match().id().update(tm_match);

            let mut state = self
                .db()
                .tab_match_state()
                .match_id()
                .find(match_id)
                .unwrap();

            state.set_live();

            self.db().tab_match_state().match_id().update(state);
        } else {
            log::error!(
                "MATCH {} WANTED TO EXIT RECOVERY SEAMLESSLY BUT WAS NOT IN RECOVERY MODE.",
                tm_match.id
            );
        }
    }

    fn match_recovery_exit_forced(&self, match_id: u32) {
        let mut tm_match = self.db().tab_match().id().find(match_id).unwrap();
        log::error!(
            "MATCH {} REALLOCATING SERVER BECAUSE RECOVERY WAS NOT SEAMLESS.",
            tm_match.id
        );

        if tm_match.is_recovery() {
            self.raw_server_occupation_remove(NodeHandle::MatchV1(match_id))
                .unwrap();

            //TODO
            //tm_match.exit_recovery();
            //self.db().tab_match().id().update(tm_match);
        }
    }

    fn match_name_edit(&self, match_id: u32, name: String) -> Result<(), String> {
        let Some(mut tm_match) = self.db().tab_match().id().find(match_id) else {
            return Err("Match not found.".into());
        };
        tm_match.name = name;
        self.db().tab_match().id().update(tm_match);

        Ok(())
    }
}

// How would a recovery flow look like?
// # Spacetime disconnects from bridge.
// -> Bridge sets match in pause mode.
// -> on_disconnect reducer runs and sets the match in recovery mode.
// ->  -> first decision because this could have been either the fault of the server itself or because the module restarted.
// -> -> Does this make a difference? -> We could insert a timer to reprovision the match? because in case of the databases fault the match should eagerly reconnect?

// Making progress is impossible if we want to allow for auto recovery.
// This is because if the raw_server caches events and continues but stays disconnected
// for an extended period of time the auto recovery could override the progress already made.
// So it is easier if we pause the match in that case.
// furthermore we would not have live data.

// If the trackmania server disconnect from the bridge we sould be able to call an reducer
// and then disconnect. This would make the reason apparent to the module but is it necessary?
// only disconnecting would do the same thing i guess

// Recovery Cases:
// - Host restarts/disconnects.
//   - rely on on_disconnected
//   - because bridge is still alive pause match
//   - Bridge eagerly tries to reconnect with seamless flag set.
//   - Upon successful reconnection we can just unpause the match.
//
// - Bridge disconnects for some other reason
//   - rely on on_disconnected
//   - because bridge is still alive pause match
//   - Bridge eagerly tries to reconnect with seamless flag set.
//   - Upon successful reconnection we can just unpause the match.
//

// Trackmania server loses connection.
// -> This case is _very_ bad.
// -> worst case is that the players keep playing -> its joever.
// -> Call reducer and that we lost connection and emergency ping to the admins or whatever.
// -> crash the whole bridge. -> restart: unless_stopped ensures eager reconnection tries.
// -> If we have a global seamless flag on the bridge which gets set to false upon start we can ensure the right connection logic.

// Now for the wombo combo.
// The bridge disconnects smh.
// Then the trackmania server also crashes in the meantime.
// We need to somehow commuicate this upon reconnection because it would be a seamless case otherise.
// This could be done via the aforementioned global seamless flag that the bridge owns.
// -> Crash the bridge so this is set to false afterwards.

// How do we handle the new match (in case it is not seamless)
// -> Need to assign a new server
// -> need to wait for players.
// -> need to load match config.
// -> need to restore the map
// -> need to restore the map progress e.g. 2 out of 4 map rounds left.
// -> need to restore the points.
// -> start. (start command would need to happen sooner i guess but idk where exactly yet.)
