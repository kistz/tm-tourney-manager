use spacetimedb::{DbContext, Local, ReducerContext, SpacetimeType, Table, reducer, table};
use tm_server_types::config::TmMode;

use crate::{
    authorization::Authorization,
    competition::{
        CompetitionPermissionsV1,
        connection::tab_connection__view,
        node::{NodeHandle, NodeWrite},
        tab_competition,
    },
    leaderboard::{filter::LbFilterSettings, merge::LbMergeSettings},
    tm_match::leaderboard::{MatchLeadearboardRead, MatchRoundPlayer},
};

mod filter;
mod merge;
//mod remap;

#[table(accessor= tab_leaderboard)]
pub struct LeaderboardV1 {
    name: String,
    settings: Vec<LbSettings>,

    #[auto_inc]
    #[primary_key]
    pub id: u32,

    #[index(hash)]
    parent_id: u32,

    template: bool,

    status: LeaderboardStatus,
}

impl LeaderboardV1 {
    pub(crate) fn instantiate(mut self, parent_id: u32, stay_template: bool) -> Self {
        self.template = stay_template;
        self.parent_id = parent_id;
        self.id = 0;
        self
    }

    pub(crate) fn is_template(&self) -> bool {
        self.template
    }

    pub(crate) fn get_comp_id(&self) -> u32 {
        self.parent_id
    }
}

#[derive(Debug, SpacetimeType, Clone, Copy, PartialEq, Eq)]
enum LeaderboardStatus {
    Configuring,
    Configured,
    //Ongoing,
    //Ended,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbSettings {
    //Remap(LbRemapSettings),
    Merge(LbMergeSettings),
    //Separate(),
    Filter(LbFilterSettings),
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum LbParams {
    Score,
    Time,
    Position,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
pub struct LbEntry {
    user_id: u32,    // non null -> always there
    position: u16, //non null -> always there -> user and position could always provide the pure lb :thinking:
    round: u16,    //nullable
    map_id: u32,   //nullable
    score: i32,    // 0 can have meaning -> yek -> i32::MIN ???
    time: i32, // 0 can have meaning? -> yes dnf or -1 is dnf -> maybe -2 is "neutral" (or i32::MIN)
    origin_idx: u16, // -> either nullable because we start with 0 or we start with 0 either way -> probably start with 1 then you can handle better
    mode: TmMode, // is there a case where we have no info about any mode? -> Yes registration -> uses uknown
    step_idx: u8, // This could serve as a deterministic replay state if you want to reason about where the score is coming from.
}

impl LbEntry {
    pub(crate) fn new(
        user_id: u32,
        mode: TmMode,
        position: u16, /* , origin_idx: u16 */
    ) -> Self {
        LbEntry {
            user_id,
            position,
            mode,
            map_id: 0,
            round: 0,
            score: i32::MIN,
            time: i32::MIN,
            origin_idx: 0, //TODO
            step_idx: 0,   //TODO
        }
    }

    pub(crate) fn set_score(mut self, score: i32) -> Self {
        self.score = score;
        self
    }

    pub(crate) fn set_time(mut self, time: i32) -> Self {
        self.time = time;
        self
    }

    pub(crate) fn set_round(mut self, round: u16) -> Self {
        self.round = round;
        self
    }
    pub(crate) fn set_map(mut self, map: u32) -> Self {
        self.map_id = map;
        self
    }

    pub(crate) fn get_user(&self) -> u32 {
        self.user_id
    }
}

#[reducer]
fn leaderboard_create(
    ctx: &ReducerContext,
    name: String,
    parent_id: u32,
    with_template: u32,
) -> Result<(), String> {
    let Some(parent_competition) = ctx.db.tab_competition().id().find(parent_id) else {
        return Err("Invalid competition".into());
    };

    ctx.auth_builder(parent_id)
        //.permission(CompetitionPermissionsV1::LEADERB)
        .authorize()?;

    if parent_competition.is_template() {
        return Err(
            "Cannot add a normal server to a template. Try do add a template server to id.".into(),
        );
    }

    //TODO validation.

    // Try to load template if provided
    if with_template != 0 {
        ctx.leaderboard_template_instantiate(with_template)?;
    } else {
        let output = LeaderboardV1 {
            name,
            id: 0,
            parent_id,
            template: false,
            status: LeaderboardStatus::Configuring,
            settings: Vec::new(),
        };

        let output = ctx.db.tab_leaderboard().try_insert(output)?;

        ctx.node_create(NodeHandle::LeaderboardV1(output.id))?;
    }

    Ok(())
}

#[reducer]
fn leaderboard_template_create(
    ctx: &ReducerContext,
    name: String,
    parent_id: u32,
) -> Result<(), String> {
    ctx.auth_builder(parent_id)
        //.permission(CompetitionPermissionsV1::MATCH_CREATE)
        .authorize()?;

    ctx.db.tab_leaderboard().try_insert(LeaderboardV1 {
        name,
        settings: Vec::new(),
        id: 0,
        parent_id,
        template: true,
        status: LeaderboardStatus::Configuring,
    })?;

    Ok(())
}

#[reducer]
fn leaderboard_configured(ctx: &ReducerContext, id: u32) -> Result<(), String> {
    let Some(mut leaderboard) = ctx.db.tab_leaderboard().id().find(id) else {
        return Err("Leaderboard was mot found!".into());
    };

    ctx.auth_builder(leaderboard.parent_id)
        .permission(CompetitionPermissionsV1::MATCH_CONFIGURE)
        .authorize()?;

    if leaderboard.status != LeaderboardStatus::Configuring {
        return Err("Leaderboard is not in configuring state".into());
    }
    leaderboard.status = LeaderboardStatus::Configured;

    ctx.db.tab_leaderboard().id().update(leaderboard);

    Ok(())
}

#[reducer]
fn leaderboard_settings_update(
    ctx: &ReducerContext,
    id: u32,
    settings: Vec<LbSettings>,
) -> Result<(), String> {
    let Some(mut leaderboard) = ctx.db.tab_leaderboard().id().find(id) else {
        return Err("Leaderboard not found.".into());
    };

    ctx.auth_builder(leaderboard.parent_id)
        //.permission(CompetitionPermissionsV1::REGISTRATION_CREATE)
        .authorize()?;

    //TODO maybe add state to reevaluation.
    //leaderboard.can_change_settings()?;

    leaderboard.settings = settings;

    ctx.db.tab_leaderboard().id().update(leaderboard);

    Ok(())
}

pub(crate) trait LeadearboardRead {
    fn leaderboard_evaluation(&self, leaderboard_id: u32) -> Vec<LbEntry>;
    fn leaderboard_final(&self, leaderboard_id: u32) -> Vec<LbEntry>;
}
impl<Db: DbContext> LeadearboardRead for Db {
    fn leaderboard_evaluation(&self, leaderboard_id: u32) -> Vec<LbEntry> {
        let Some(lb) = self
            .db_read_only()
            .tab_leaderboard()
            .id()
            .find(leaderboard_id)
        else {
            log::warn!("Leaderboard was evaluated which does not exist.");
            return Vec::new();
        };

        if lb.status == LeaderboardStatus::Configuring {
            log::warn!("Tried to eval leadarboard but it is still getting configured.");
            return Vec::new();
        }

        let settings = lb.settings;

        let dependencies = self
            .db_read_only()
            .tab_connection()
            .origins_of()
            .filter(NodeHandle::LeaderboardV1(leaderboard_id).split())
            .filter(|c| c.is_data());

        let Some(first_setting) = settings.get(0) else {
            log::warn!("Tried to evaluate leaderboard but it does not have settings");
            return Vec::new();
        };

        let mut leaderboard = Vec::new();

        let mut dep_len = 0;

        for depending_connection in dependencies {
            if dep_len > 1 && !matches!(first_setting, LbSettings::Merge(_)) {
                log::error!(
                    "There were more than one data connection into the leaderboard and no merge was selected"
                );
                return Vec::new();
            }

            //TODO this should probably not be handled by this function :thinking:
            match depending_connection.origin() {
                NodeHandle::MatchV1(m) => leaderboard.extend(self.match_rounds(m)),
                NodeHandle::LeaderboardV1(l) => leaderboard.extend(self.leaderboard_evaluation(l)),
                //TODO handle rest of the cases: Input/Output/Competition should be possible since they can passthrough other stuff.
                _ => {
                    log::error!("Tried to fetch a leadarboard of the wrong node.");
                    return Vec::new();
                }
            };

            dep_len += 1;
        }

        for (index, setting) in settings.into_iter().enumerate() {
            leaderboard = match setting {
                //LbSettings::Remap(lb_remap_settings) => todo!(), //lb_remap_settings.evaluate(leaderboard),
                LbSettings::Merge(lb_merge_settings) => lb_merge_settings.evaluate(leaderboard),
                LbSettings::Filter(lb_filter_settings) => lb_filter_settings.evaluate(leaderboard),
            }
        }

        log::info!("Leaderbaord {leaderboard_id}: {:?}", leaderboard);

        leaderboard
    }

    fn leaderboard_final(&self, leaderboard_id: u32) -> Vec<LbEntry> {
        let leaderboard = self.leaderboard_evaluation(leaderboard_id);
        if leaderboard.is_empty() {
            log::error!("leaderboard returned empty vec.");
            return Vec::new();
        }
        //TODO

        leaderboard
    }
}
pub(crate) trait LeaderboardWrite: LeadearboardRead {
    fn leaderboard_template_instantiate(&self, with_template: u32) -> Result<(), String>;
    fn leaderboard_insert(&self, output: LeaderboardV1) -> Result<LeaderboardV1, String>;
    fn leaderboard_name_edit(&self, leaderboard_i32: u32, name: String) -> Result<(), String>;
}
impl<Db: DbContext<DbView = Local>> LeaderboardWrite for Db {
    fn leaderboard_template_instantiate(&self, with_template: u32) -> Result<(), String> {
        todo!()
    }

    fn leaderboard_insert(&self, output: LeaderboardV1) -> Result<LeaderboardV1, String> {
        todo!()
    }

    fn leaderboard_name_edit(&self, leaderboard_id: u32, name: String) -> Result<(), String> {
        let Some(mut tm_match) = self.db().tab_leaderboard().id().find(leaderboard_id) else {
            return Err("Match not found.".into());
        };
        tm_match.name = name;
        self.db().tab_leaderboard().id().update(tm_match);

        Ok(())
    }
}

// We should be able to iterate over every input and accumulate score or position.
//After the accumulation there shuold also be math operations possible.

// How would a distribution onto two servers work?
// would require a 50/50 rotating live distribution of players

/* trait ModifiableLeaderboard {
    fn evaluate(self, input: Vec<MatchRoundPlayer>) -> Vec<MatchRoundPlayer>;
} */

// Leadarboard should be implicitly constrained to a specific player.
// this is important because a player is its own "entity" and it makes no sense
// to have one in a leadearboard multiple times.

// Matches have two "channels" rounds leadarboard and match leadarboard.
// The match leadarboard is only virtually constructed.
// Does this mean the leaderboard node should do the same??? -> i guess ja
// This would also be very good then mhm.

// -> we nonoetheless need to think of new types which we can remap between.
// -> this implies that nothing is materialized so no inndices are possible on SpacetimeType.

// The connetion filtering is applied to the match_leadarboard.
// This means in order to apply the filter input we would have to evaluate the match leadarboard
// and then get the players which are currently in front and remap it to the rounds again.
// do we want to allow that or not? -> rn it would be alriiiight??
//

// all of the above also means that upon multiple input connections you HAVE to merge them together in the first setting.

/* struct LbWrapper {
    inner: LbEntryV2,
} */

/* enum LbEntryV2 {
    Matches(Vec<Vec<{}>>),
    Pure(Vec<{user,position, score,time}) //-> would be possible stale aswell....

} */

// we have the 0 to mark stale things :thinking:

/* pub(crate) trait LeadearboardOperations {

}

impl LeadearboardOperations for Vec<LbEntry> {
    fn purify(&mut self) {
        self.
    }
} */
