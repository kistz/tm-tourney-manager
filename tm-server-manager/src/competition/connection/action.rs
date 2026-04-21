use spacetimedb::{ReducerContext, SpacetimeType, table};

use crate::{
    competition::node::NodeHandle,
    registration::RegistrationWrite,
    tm_match::{match_try_start, tab_match},
};

#[table(accessor=tab_connection_action)]
pub struct TabConnectionAction {
    #[index(hash)]
    competition_id: u32,
    #[primary_key]
    pub connection_id: u32,

    action: ConnectionAction,
}

impl TabConnectionAction {
    pub(super) fn new(
        target: NodeHandle,
        parent_id: u32,
        connection_id: u32,
    ) -> Result<Self, String> {
        Ok(Self {
            competition_id: parent_id,
            connection_id,
            action: match target {
                NodeHandle::MatchV1(_) => {
                    ConnectionAction::MatchV1(ConnectionActionMatch::TryStart)
                }
                NodeHandle::CompetitionV1(_) => unreachable!(),
                NodeHandle::ScheduleV1(_) => unreachable!(),
                NodeHandle::ServerV1(_) => unreachable!(),
                NodeHandle::InputV1(_) => unreachable!(),
                NodeHandle::OutputV1(_) => unreachable!(),
                NodeHandle::RegistrationV1(_) => {
                    ConnectionAction::RegistrationV1(ConnectionActionRegistration::Open)
                }
            },
        })
    }

    fn get_match(&self) -> ConnectionActionMatch {
        match self.action {
            ConnectionAction::MatchV1(connection_action_match) => connection_action_match,
            _ => unreachable!(),
        }
    }

    fn get_registration(&self) -> ConnectionActionRegistration {
        match self.action {
            ConnectionAction::RegistrationV1(action) => action,
            _ => unreachable!(),
        }
    }
}

// Versioning works be e.g.:
// MatchV1A2(ConnectionActionMatchV2)
#[derive(Debug, SpacetimeType)]
enum ConnectionAction {
    MatchV1(ConnectionActionMatch),
    RegistrationV1(ConnectionActionRegistration),
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum ConnectionActionMatch {
    TryStart,
    ForceStart,
}

#[derive(Debug, SpacetimeType, Clone, Copy)]
enum ConnectionActionRegistration {
    Open,
    Close,
}

pub(super) fn try_exec_action(connection: u32, target: NodeHandle, ctx: &ReducerContext) {
    let action = ctx
        .db
        .tab_connection_action()
        .connection_id()
        .find(connection)
        .unwrap();
    match target {
        NodeHandle::MatchV1(m) => {
            let match_action = action.get_match();
            match match_action {
                ConnectionActionMatch::TryStart => {
                    if let Err(error) = match_try_start(ctx, m) {
                        log::error!(
                            "Explicit Flow: match_try_start action failed through connection {} Error: {}",
                            connection,
                            error
                        );
                    }
                }
                ConnectionActionMatch::ForceStart => todo!(),
            }
        }
        NodeHandle::CompetitionV1(_) => unreachable!(),
        NodeHandle::ServerV1(_) => unreachable!(),
        NodeHandle::ScheduleV1(_) => unreachable!(),
        NodeHandle::RegistrationV1(r) => {
            let registration_action = action.get_registration();
            match registration_action {
                ConnectionActionRegistration::Open => {
                    if let Err(err) = ctx.registration_open(r) {
                        log::error!(
                            "Explicit Flow: registration_open action failed through connection {} Error: {}",
                            connection,
                            err
                        );
                    }
                }
                ConnectionActionRegistration::Close => {
                    if let Err(err) = ctx.registration_close(r) {
                        log::error!(
                            "Explicit Flow: registration_close action failed through connection {} Error: {}",
                            connection,
                            err
                        );
                    }
                }
            }
        }
        NodeHandle::InputV1(_) => unreachable!(),
        NodeHandle::OutputV1(_) => unreachable!(),
    }
}
