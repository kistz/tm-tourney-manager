use spacetimedb::{
    AnonymousViewContext, Query, RawQuery, ReducerContext, Table, Timestamp, Uuid, reducer, table,
    view,
};

use crate::{
    authorization::Authorization,
    registration::{RegistrationStatus, tab_registration},
};

//TODO make this table private again.
#[table(
    accessor=tab_registeration_player,
    index(accessor=user_registered, hash(columns=[user_id,registration_id])),
    public
)]
#[derive(Debug, Clone, Copy)]
pub struct RegisterationPlayer {
    pub registered_at: Timestamp,
    #[index(hash)]
    pub registration_id: u32,
    pub user_id: u32,
}

#[view(accessor=temp_registration_player,public)]
fn temp_registration_player(
    ctx: &AnonymousViewContext, /* ,registration_id: u32 */
) -> Vec<RegisterationPlayer> {
    let registration_id = 2u32;
    ctx.registration_player(registration_id)
}

/* #[view(accessor=unstable_registration_player,public)]
fn unstable_registration_player(ctx: &AnonymousViewContext) -> impl Query<RegisterationPlayer> {
    //ctx.from.tab_registeration_player()
    RawQuery::<RegisterationPlayer>::new(String::new())
} */

#[reducer]
fn register_player(ctx: &ReducerContext, registration_id: u32) -> Result<(), String> {
    let user_id = ctx.user_id()?;

    let Some(registration) = ctx.db.tab_registration().id().find(registration_id) else {
        return Err("Tried to register but the registration id does not exist.".into());
    };

    registration.player_registration_allowed(ctx)?;

    if ctx
        .db
        .tab_registeration_player()
        .user_registered()
        .filter((user_id, registration_id))
        .count()
        != 0
    {
        return Err("User is already registered for registration_id!".to_string());
    }

    ctx.db
        .tab_registeration_player()
        .try_insert(RegisterationPlayer {
            registration_id,
            user_id,
            registered_at: ctx.timestamp,
        })?;

    Ok(())
}

#[reducer]
fn unregister_player(ctx: &ReducerContext, registration_id: u32) -> Result<(), String> {
    let account_id = ctx.user_id()?;

    let Some(registration) = ctx.db.tab_registration().id().find(registration_id) else {
        return Err("Tried to register for a competition that doesnt exist.".into());
    };

    if registration.status != RegistrationStatus::Ongoing {
        return Err("Registration not active".into());
    }

    let Some(registred_user) = ctx
        .db
        .tab_registeration_player()
        .registration_id()
        .filter(registration_id)
        .find(|p| p.user_id == account_id)
    else {
        return Err("User is not registered for competition!".to_string());
    };

    if !ctx.db.tab_registeration_player().delete(registred_user) {
        return Err(format!(
            "Unexpected error occured deleting the user {} from {}",
            account_id, registration_id
        ));
    };

    Ok(())
}

pub(crate) trait RegistrationRead {
    fn registration_player(&self, registration_id: u32) -> Vec<RegisterationPlayer>;
}
impl<Db: spacetimedb::DbContext> RegistrationRead for Db {
    fn registration_player(&self, registration_id: u32) -> Vec<RegisterationPlayer> {
        let mut registered = self
            .db_read_only()
            .tab_registeration_player()
            .registration_id()
            .filter(registration_id)
            .collect::<Vec<_>>();

        registered.sort_by_key(|f| f.registered_at);

        registered
    }
}
