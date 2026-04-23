use spacetimedb::{DbContext, Local, ReducerContext, Table, reducer, table};

use crate::{
    authorization::Authorization,
    competition::{
        CompetitionPermissionsV1,
        node::{NodeHandle, NodeWrite},
        tab_competition,
    },
};

#[table(accessor= tab_leaderboard)]
pub struct LeaderboardV1 {
    name: String,

    #[auto_inc]
    #[primary_key]
    pub id: u32,

    #[index(hash)]
    parent_id: u32,

    template: bool,
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
        };

        let output = ctx.db.tab_leaderboard().try_insert(output)?;

        ctx.node_create(NodeHandle::LeaderboardV1(output.id))?;
    }

    Ok(())
}

/* pub(crate) trait OutputRead {
    fn outputs_in_parent(&self, parent_id: u32) -> impl Iterator<Item = OutputV1>;
}
impl<Db: DbContext> OutputRead for Db {
    fn outputs_in_parent(&self, parent_id: u32) -> impl Iterator<Item = OutputV1> {
        self.db_read_only()
            .tab_output()
            .parent_id()
            .filter(parent_id)
    }
} */
pub(crate) trait LeaderboardWrite {
    fn leaderboard_template_instantiate(&self, with_template: u32) -> Result<(), String>;
    fn leaderboard_insert(&self, output: LeaderboardV1) -> Result<LeaderboardV1, String>;
    fn leaderboard_name_edit(&self, leadaerboard_i32: u32, name: String) -> Result<(), String>;
}
impl<Db: DbContext<DbView = Local>> LeaderboardWrite for Db {
    fn leaderboard_template_instantiate(&self, with_template: u32) -> Result<(), String> {
        todo!()
    }

    fn leaderboard_insert(&self, output: LeaderboardV1) -> Result<LeaderboardV1, String> {
        todo!()
    }

    fn leaderboard_name_edit(&self, leadaerboard_id: u32, name: String) -> Result<(), String> {
        let Some(mut tm_match) = self.db().tab_leaderboard().id().find(leadaerboard_id) else {
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
