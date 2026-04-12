use base64::Engine;
use base64::prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD};
use petgraph::visit::Time;
use serde::Deserialize;
use spacetimedb::http::Request;
use spacetimedb::{
    DbContext, Identity, Local, ReducerContext, Table, Timestamp, Uuid, ViewContext, reducer, table,
};
use spacetimedb::{ProcedureContext, view};

use crate::authorization::Authorization;
use crate::competition::node::{NodeHandle, NodeRead};
use crate::competition::server_pool::TabCompetitionServerPoolRead;
use crate::raw_server::occupation::{TabRawServerOccupationRead, TabRawServerOccupationWrite};
use crate::tm_match::state::tab_match_state;
use crate::tm_match::{MatchWrite, tab_match};
use crate::user::UserRead;

pub mod config;
pub mod destination;
pub mod event;
pub mod method;
pub mod occupation;
pub mod player;
pub mod replay;

#[spacetimedb::table(accessor=tab_raw_server)]
pub struct RawServerV1 {
    #[unique]
    pub identity: Identity,
    #[unique]
    pub server_login: String,

    // Account id of the server from the trackmania web services.
    server_account_id: Uuid,

    // Whenever the server has last connected or disconnected.
    // Read this together with online or offline to get the correct value.
    last_connection: Timestamp,

    /// Each server also has a ubisoft account associated with it.
    /// This is a user_account_id because you could add a user which was not seen yet.
    #[index(hash)]
    pub(crate) user_id: u32,

    #[auto_inc]
    #[primary_key]
    pub(crate) id: u32,

    // Whether the server can be reached with a bridge active.
    online: bool,

    // Can the server be provisioned or is it a fixed server?
    capturable: bool,

    // This is necessary because at the moment a arbitrary account_id can be supplied when logging in as a server
    // as there is no way to verify it through the trackmania web services.
    // To avoid adding servers to a the pool of a user without verification (which could be an attack vector) we require manual verification from the user.
    verified: bool,
}

impl RawServerV1 {
    pub fn set_online(&mut self, when: Timestamp) {
        self.last_connection = when;
        self.online = true;
    }
    pub fn set_offline(&mut self, when: Timestamp) {
        self.last_connection = when;
        self.online = false;
    }

    pub fn set_identity(&mut self, identity: Identity) {
        self.identity = identity;
    }

    pub fn is_verified(&self) -> bool {
        self.verified
    }
}

/// Elevates an annonymous connection to a trackmania dedicated server sidecar.
/// password of the server doesn't get saved but rather verified for validity.
#[spacetimedb::procedure]
pub fn login_as_server(
    ctx: &mut ProcedureContext,
    login: String,
    password: String,
    user_account_id: Uuid,
    seamless: bool,
) -> Result<u32, String> {
    let request = Request::builder()
        .method("POST")
        .uri("https://prod.trackmania.core.nadeo.online/v2/authentication/token/basic")
        .header(
            "Authorization",
            format!(
                "Basic {}",
                BASE64_STANDARD.encode(login.clone() + ":" + &password)
            ),
        )
        .header("Content-Type", "application/json")
        .header("User-Agent", "tm-server-manager | central")
        .body(r#"{ "audience": "NadeoServices" }"#)
        .map_err(|e| e.to_string())?;
    let result = ctx
        .http
        .send(request)
        .map_err(|e| format!("Internal Error! The HTTP request could not be sent! Error: {e}"))?;

    let status = result.status();

    if !status.is_success() {
        log::error!("Login attempt from server ({}) was not a success", login);
        return Err("Server registration failed because credential were wrong".into());
    }

    #[derive(Debug, Deserialize)]
    #[allow(non_snake_case)]
    struct NadeoServerToken {
        accessToken: String,
    }

    #[derive(Debug, Deserialize)]
    struct NadeoServerClaims {
        sub: String,
    }

    let mut body_string = result.into_body().into_string_lossy();

    let token =
        unsafe { json::from_str::<NadeoServerToken>(&mut body_string).map_err(|e| e.to_string())? };
    let payload = token.accessToken.split(".").collect::<Vec<_>>()[1].to_string();
    let mut payload = BASE64_URL_SAFE_NO_PAD.decode(payload).unwrap();
    let claims = json::from_slice::<NadeoServerClaims>(&mut payload).map_err(|e| e.to_string())?;

    let server_account_id = Uuid::parse_str(&claims.sub).unwrap();
    let identity = ctx.sender();

    let id = ctx.try_with_tx::<u32, String>(|ctx| {
        if let Some(mut server) = ctx.db.tab_raw_server().server_login().find(&login) {
            let server_id = server.id;
            if ctx.user_account_from_id(server.user_id) != user_account_id {
                server.verified = false;
            }
            server.set_online(ctx.timestamp);
            server.set_identity(identity);
            ctx.db.tab_raw_server().id().update(server);

            //RECOVERY
            if let Some(occupation) = ctx.raw_server_occupation(server_id)
                && occupation.is_match()
            {
                let match_id = occupation.id();

                if seamless {
                    ctx.match_recovery_exit_seamless(match_id);
                } else {
                    ctx.match_recovery_exit_forced(match_id);
                }
            }
            Ok(server_id)
        } else {
            // Server has never been seen before so create a new one.
            let server = ctx.db.tab_raw_server().try_insert(RawServerV1 {
                id: 0,
                server_login: login.clone(),
                server_account_id,
                user_id: ctx.user_id_from_account(user_account_id),
                identity,
                capturable: true,
                verified: false,
                online: true,
                last_connection: ctx.timestamp,
            })?;
            Ok(server.id)
        }
    })?;

    Ok(id)
}

#[view(accessor= this_raw_server, public)]
fn this_raw_server(ctx: &ViewContext) -> Option<RawServerV1> {
    ctx.db.tab_raw_server().identity().find(ctx.sender())
}

/// The Raw server pool are all servers of an account which are verified.
#[view(accessor= user_raw_server_pool, public)]
pub(crate) fn user_raw_server_pool(ctx: &ViewContext) -> Vec<RawServerV1> {
    let Ok(user_id) = ctx.user_id() else {
        return Vec::new();
    };
    //TODO maybe switch to query builder if possible
    ctx.db
        .tab_raw_server()
        .user_id()
        .filter(user_id)
        .filter(|s| s.verified)
        .collect()
}

/// The Raw server pool are all servers of an account which are verified.
#[view(accessor= user_available_server_pool, public)]
pub(crate) fn user_available_server_pool(ctx: &ViewContext) -> Vec<RawServerV1> {
    let Ok(user_id) = ctx.user_id() else {
        return Vec::new();
    };

    ctx.db
        .tab_raw_server()
        .user_id()
        .filter(user_id)
        .filter(|s| s.verified && s.capturable)
        .filter(|s| !ctx.raw_server_is_occupied(s.id))
        .collect()
}

/// The unverified version of a server pool includes all servers of an account which are not vet verified.
#[view(accessor= user_raw_server_pool_unverified, public)]
fn user_raw_server_pool_unverified(ctx: &ViewContext) -> Vec<RawServerV1> {
    let Ok(user_id) = ctx.user_id() else {
        return Vec::new();
    };
    //TODO maybe switch to query builder if possible
    ctx.db
        .tab_raw_server()
        .user_id()
        .filter(user_id)
        .filter(|s| !s.verified)
        .collect()
}

#[reducer]
fn raw_server_verify(ctx: &ReducerContext, server_id: u32) -> Result<(), String> {
    let user_id = ctx.user_id()?;

    let mut server = ctx
        .db
        .tab_raw_server()
        .id()
        .find(server_id)
        .ok_or("Couldnt find server with login")?;

    if server.user_id == user_id {
        if server.verified {
            Err("Server was already verified.".into())
        } else {
            server.verified = true;
            ctx.db.tab_raw_server().id().update(server);
            Ok(())
        }
    } else {
        Err("Not permitted to edit the server".into())
    }
}

pub(crate) trait TabRawServerRead {
    fn raw_server_last_connection(&self, server_id: u32) -> Timestamp;
}
pub(crate) trait TabRawServerWrite: TabRawServerRead {
    fn raw_server_pool_assign(&self, node_handle: NodeHandle) -> Result<u32, String>;
    fn raw_server_disconnected(&self, server: RawServerV1, now: Timestamp);
}

impl<Db: DbContext> TabRawServerRead for Db {
    fn raw_server_last_connection(&self, server_id: u32) -> Timestamp {
        self.db_read_only()
            .tab_raw_server()
            .id()
            .find(server_id)
            .unwrap()
            .last_connection
    }
}

impl<Db: DbContext<DbView = Local>> TabRawServerWrite for Db {
    fn raw_server_pool_assign(&self, node_handle: NodeHandle) -> Result<u32, String> {
        let available_servers = self.server_pool_available(self.node_get_parent(node_handle)?);
        if available_servers.is_empty() {
            return Err("No server is assigned to the match and there are no servers left to auto provision. Cannot start the match!".into());
        }

        let server_id = available_servers[0].id;

        self.raw_server_occupation_add(node_handle, server_id)?;

        Ok(server_id)
    }

    fn raw_server_disconnected(&self, mut server: RawServerV1, when: Timestamp) {
        let server_id = server.id;

        server.set_offline(when);
        self.db().tab_raw_server().id().update(server);

        if let Some(occupation) = self.raw_server_occupation(server_id)
            && occupation.is_match()
        {
            let match_id = occupation.id();

            self.match_recovery_enter(match_id);
        }
    }
}

//mod huh;
