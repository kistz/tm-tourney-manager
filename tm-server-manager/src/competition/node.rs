use std::collections::HashMap;

use spacetimedb::{DbContext, Local, ReducerContext, SpacetimeType, Uuid, reducer};

use crate::{
    competition::{
        CompetitionPermissionsV1, CompetitionRead, CompetitionWrite,
        connection::{
            ConnectionRead, action::tab_connection_action, data::tab_connection_data,
            tab_connection, tab_connection__view,
        },
        roles::tab_competition_member__view,
        tab_competition, tab_competition__view,
    },
    input::{InputWrite, tab_input, tab_input__view},
    output::{OutputWrite, tab_output, tab_output__view},
    raw_server::player::PermittedPlayer,
    registration::{RegistrationWrite, tab_registration, tab_registration__view},
    schedule::{ScheduleWrite, tab_schedule, tab_schedule__view},
    tm_match::{MatchWrite, tab_match, tab_match__view},
    tm_server::{ServerWrite, tab_server, tab_server__view},
    user::UserRead,
};
mod position;

pub use position::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, SpacetimeType, Hash)]
#[non_exhaustive]
pub enum NodeHandle {
    MatchV1(u32),
    CompetitionV1(u32),
    ScheduleV1(u32),
    ServerV1(u32),
    InputV1(u32),
    OutputV1(u32),
    RegistrationV1(u32),
    LeaderboardV1(u32),
}

// This is done because of a petgraph trait bound.
impl Default for NodeHandle {
    fn default() -> Self {
        log::error!(
            "Tried to call the deafault implementation of NodeKindHandle.
            This should not be possible and is only implemented because of a petgraph trait bound."
        );
        panic!()
    }
}

impl NodeHandle {
    pub(crate) fn split(self) -> (u8, u32) {
        match self {
            NodeHandle::MatchV1(m) => (1, m),
            NodeHandle::CompetitionV1(c) => (2, c),
            NodeHandle::ScheduleV1(s) => (3, s),
            NodeHandle::ServerV1(m) => (4, m),
            NodeHandle::InputV1(h) => (5, h),
            NodeHandle::OutputV1(h) => (6, h),
            NodeHandle::RegistrationV1(r) => (7, r),
            NodeHandle::LeaderboardV1(l) => (8, l),
        }
    }

    pub(crate) fn combine(variant: u8, value: u32) -> Self {
        match variant {
            1 => Self::MatchV1(value),
            2 => Self::CompetitionV1(value),
            3 => Self::ScheduleV1(value),
            4 => Self::ServerV1(value),
            5 => Self::InputV1(value),
            6 => Self::OutputV1(value),
            7 => Self::RegistrationV1(value),
            8 => Self::LeaderboardV1(value),
            _ => unreachable!(),
        }
    }

    pub(crate) fn is_template(&self, ctx: &ReducerContext) -> bool {
        match self {
            NodeHandle::MatchV1(m) => {
                let node = ctx.db.tab_match().id().find(m).unwrap();
                node.is_template()
            }
            NodeHandle::CompetitionV1(c) => {
                let node = ctx.db.tab_competition().id().find(c).unwrap();
                node.is_template()
            }
            NodeHandle::ScheduleV1(s) => {
                let node = ctx.db.tab_schedule().id().find(s).unwrap();
                node.is_template()
            }
            NodeHandle::ServerV1(n) => {
                let node = ctx.db.tab_server().id().find(n).unwrap();
                node.is_template()
            }
            NodeHandle::RegistrationV1(reg) => {
                let node = ctx.db.tab_registration().id().find(reg).unwrap();
                node.is_template()
            }
            NodeHandle::InputV1(n) => {
                let node = ctx.db.tab_input().id().find(n).unwrap();
                node.is_template()
            }
            NodeHandle::OutputV1(n) => {
                let node = ctx.db.tab_output().id().find(n).unwrap();
                node.is_template()
            }
            NodeHandle::LeaderboardV1(n) => {
                /* let node = ctx.db.tab_leaderboard().id().find(n).unwrap();
                node.is_template() */
                todo!()
            }
        }
    }

    pub(crate) fn is_match(&self) -> bool {
        matches!(self, NodeHandle::MatchV1(_))
    }
    pub(crate) fn is_server(&self) -> bool {
        matches!(self, NodeHandle::ServerV1(_))
    }
    pub(crate) fn is_input(&self) -> bool {
        matches!(self, NodeHandle::InputV1(_))
    }
    pub(crate) fn is_output(&self) -> bool {
        matches!(self, NodeHandle::OutputV1(_))
    }
    pub(crate) fn is_competition(&self) -> bool {
        matches!(self, NodeHandle::CompetitionV1(_))
    }
    pub(crate) fn is_registration(&self) -> bool {
        matches!(self, NodeHandle::RegistrationV1(_))
    }
    pub(crate) fn is_schedule(&self) -> bool {
        matches!(self, NodeHandle::ScheduleV1(_))
    }
    pub(crate) fn is_leaderboard(&self) -> bool {
        matches!(self, NodeHandle::LeaderboardV1(_))
    }

    pub(crate) fn id(&self) -> u32 {
        self.split().1
    }
}

/* pub trait NodeType {
    fn ready(&self, ctx: &ReducerContext) -> Result<(), String>;
}

impl NodeType for NodeHandle {
    fn ready(&self, ctx: &ReducerContext) -> Result<(), String> {
        match self {
            NodeHandle::MatchV1(match_id) => ctx.match_set_preparation(*match_id, ctx.timestamp),
            NodeHandle::CompetitionV1(c) => todo!(), // trigger the input node.
            NodeHandle::ServerV1(_) => todo!(),
            NodeHandle::ScheduleV1(s) => ctx.schedule_start_relative(*s, ctx.timestamp),
            NodeHandle::RegistrationV1(r) => unreachable!(),
            NodeHandle::InputV1(_) => todo!(),
            NodeHandle::OutputV1(_) => todo!(),
            // Nothing should really happen here because the leaderboard will recompute itself.
            NodeHandle::LeaderboardV1(_) => Ok(()),
        }
    }
} */

pub(crate) trait NodeRead {
    fn node_permitted_players_input(&self, node: NodeHandle) -> Vec<PermittedPlayer>;
    fn node_get_parent(&self, node: NodeHandle) -> Result<u32, String>;
}
impl<Db: DbContext> NodeRead for Db {
    fn node_permitted_players_input(&self, node: NodeHandle) -> Vec<PermittedPlayer> {
        let mut map: HashMap<Uuid, PermittedPlayer> = HashMap::new();

        let parent = self.node_get_parent(node).unwrap();

        let tree = self.competition_ancestors(parent);
        for comp in tree {
            map.extend(
                self.db_read_only()
                    .tab_competition_member()
                    .competition_id()
                    .filter(comp)
                    .filter_map(|m| {
                        if m.get_permissions()
                            .has(CompetitionPermissionsV1::TRACKMANIA_SPECTATE_MATCHES)
                        {
                            let account_id = self.user_account_from_id(m.user());
                            return Some((
                                account_id,
                                PermittedPlayer::new(account_id, false, true),
                            ));
                        }
                        None
                    }),
            )
        }

        let depending_connections = self
            .db_read_only()
            .tab_connection()
            .origins_of()
            .filter(node.split())
            .filter(|c| c.is_data());

        for depending_connection in depending_connections {
            let permitted_players = self
                .connection_filter_permitted_players(depending_connection)
                .into_iter()
                .map(|p| (p.account_id, p));
            // This overrides the existing entrys.
            map.extend(permitted_players);
        }

        map.into_values().collect()
    }

    fn node_get_parent(&self, node: NodeHandle) -> Result<u32, String> {
        match node {
            NodeHandle::MatchV1(m) => {
                if let Some(ma) = self.db_read_only().tab_match().id().find(m) {
                    Ok(ma.get_comp_id())
                } else {
                    Err("Match couldnt be found.".into())
                }
            }
            NodeHandle::CompetitionV1(c) => {
                if let Some(co) = self.db_read_only().tab_competition().id().find(c) {
                    let id = co.id;
                    if id != 0 {
                        Ok(id)
                    } else {
                        Err("Compeittion without Parent cannot be part of a connection".into())
                    }
                } else {
                    Err("Competition could not be found".into())
                }
            }
            NodeHandle::ScheduleV1(s) => {
                if let Some(ma) = self.db_read_only().tab_schedule().id().find(s) {
                    Ok(ma.parent_id())
                } else {
                    Err("Schedule could not be found.".into())
                }
            }
            NodeHandle::ServerV1(s) => {
                if let Some(ma) = self.db_read_only().tab_server().id().find(s) {
                    Ok(ma.parent_id())
                } else {
                    Err("Server could not be found.".into())
                }
            }
            NodeHandle::RegistrationV1(reg) => {
                if let Some(reg) = self.db_read_only().tab_registration().id().find(reg) {
                    Ok(reg.get_comp_id())
                } else {
                    Err("Registration could not be found.".into())
                }
            }
            NodeHandle::InputV1(node) => {
                if let Some(node) = self.db_read_only().tab_input().id().find(node) {
                    Ok(node.get_comp_id())
                } else {
                    Err("Registration could not be found.".into())
                }
            }
            NodeHandle::OutputV1(node) => {
                if let Some(node) = self.db_read_only().tab_output().id().find(node) {
                    Ok(node.get_comp_id())
                } else {
                    Err("Registration could not be found.".into())
                }
            }
            NodeHandle::LeaderboardV1(node) => {
                /* if let Some(node) = self.db_read_only().tab_registration().id().find(node) {
                    Ok(node.get_comp_id())
                } else {
                    Err("Registration could not be found.".into())
                } */
                todo!()
            }
        }
    }
}

pub(crate) trait NodeWrite: NodeRead {
    fn node_create(&self, node: NodeHandle) -> Result<(), String>;
    fn node_delete(&self, node: NodeHandle) -> Result<(), String>;
    fn node_name_edit(&self, node: NodeHandle, name: String) -> Result<(), String>;
}
impl<Db: DbContext<DbView = Local>> NodeWrite for Db {
    fn node_create(&self, node: NodeHandle) -> Result<(), String> {
        self.node_position_insert(node)?;

        Ok(())
    }

    fn node_delete(&self, node: NodeHandle) -> Result<(), String> {
        // Delete postition table entry.
        self.node_position_delete(node);

        // Delete all associated connection tables involving the node.
        let connections = self
            .db()
            .tab_connection()
            .origins_of()
            .filter(node.split())
            .chain(self.db().tab_connection().targets_of().filter(node.split()));
        for connection in connections {
            if connection.is_action() {
                self.db()
                    .tab_connection_action()
                    .connection_id()
                    .delete(connection.id);
                continue;
            }
            if connection.is_data() {
                self.db()
                    .tab_connection_data()
                    .connection_id()
                    .delete(connection.id);
                continue;
            }
        }
        self.db().tab_connection().origins_of().delete(node.split());
        self.db().tab_connection().targets_of().delete(node.split());

        Ok(())
    }

    fn node_name_edit(&self, node: NodeHandle, name: String) -> Result<(), String> {
        match node {
            NodeHandle::MatchV1(id) => self.match_name_edit(id, name)?,
            NodeHandle::CompetitionV1(id) => self.competition_name_edit(id, name)?,
            NodeHandle::ServerV1(id) => self.server_name_edit(id, name)?,
            NodeHandle::ScheduleV1(id) => self.schedule_name_edit(id, name)?,
            NodeHandle::RegistrationV1(id) => self.registration_name_edit(id, name)?,
            NodeHandle::InputV1(id) => self.input_name_edit(id, name)?,
            NodeHandle::OutputV1(id) => self.output_name_edit(id, name)?,
            NodeHandle::LeaderboardV1(id) => todo!(), //self.leaderboard_name_edit(id, name)?,
        }

        Ok(())
    }
}

#[reducer]
fn node_name_edit(ctx: &ReducerContext, node: NodeHandle, name: String) -> Result<(), String> {
    //TODO access control

    ctx.node_name_edit(node, name)
}
