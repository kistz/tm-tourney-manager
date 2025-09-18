use spacetimedb::{
    reducer, table, Identity, ReducerContext, ScheduleAt, SpacetimeType, Table, TimeDuration,
    Timestamp,
};
use tm_server_types::WayPointEvent;

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: Option<String>,
    online: bool,
}

#[table(name = message, public)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String,
}

#[derive(Debug, SpacetimeType)]
pub enum Roles {}

pub struct Tournament {}

#[table(name = server)]
pub struct Server {
    server_id: String,

    owner: User,
    events: ServerEvents,

    status: ServerStatus,
}

#[derive(Debug, SpacetimeType)]
pub struct ServerStatus {}

#[table(name = server_events,public)]
pub struct ServerEvents {
    event: String,
    content: String,
    typed: WayPointEvent,
}

#[reducer]
pub fn post_event(ctx: &ReducerContext, event: WayPointEvent) {
    log::info!("{event:?}");
    ctx.db.server_events().insert(ServerEvents {
        event: "jaaa".to_string(),
        content: "".to_owned(),
        typed: event,
    });
}

#[reducer]
/// Clients invoke this reducer to set their user names.
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            name: Some(name),
            ..user
        });
        Ok(())
    } else {
        Err("Cannot set name for unknown userd".to_string())
    }
}

/// Takes a name and checks if it's acceptable as a user's name.
fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Names must not be empty".to_string())
    } else {
        Ok(name)
    }
}

#[reducer]
/// Clients invoke this reducer to send messages.
pub fn send_message(ctx: &ReducerContext, text: String) -> Result<(), String> {
    let text = validate_message(text)?;
    log::info!("{}", text);
    ctx.db.message().insert(Message {
        sender: ctx.sender,
        text,
        sent: ctx.timestamp,
    });

    /* ctx.db.send_message_schedule().insert(SendMessageSchedule {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Time(ctx.timestamp + TimeDuration::from_micros(10000)),
        text: "What the helly".into(),
    }); */
    Ok(())
}

/// Takes a message's text and checks if it's acceptable to send.
fn validate_message(text: String) -> Result<String, String> {
    if text.is_empty() {
        Err("Messages must not be empty".to_string())
    } else {
        Ok(text)
    }
}

#[reducer(client_connected)]
// Called when a client connects to a SpacetimeDB database server
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        // If this is a returning user, i.e. we already have a `User` with this `Identity`,
        // set `online: true`, but leave `name` and `identity` unchanged.
        ctx.db.user().identity().update(User {
            online: true,
            ..user
        });
    } else {
        // If this is a new user, create a `User` row for the `Identity`,
        // which is online, but hasn't set a name.
        ctx.db.user().insert(User {
            name: None,
            identity: ctx.sender,
            online: true,
        });
    }
}

#[reducer(client_disconnected)]
// Called when a client disconnects from SpacetimeDB database server
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            online: false,
            ..user
        });
    } else {
        // This branch should be unreachable,
        // as it doesn't make sense for a client to disconnect without connecting first.
        log::warn!(
            "Disconnect event for unknown user with identity {:?}",
            ctx.sender
        );
    }
}

#[table(name = send_message_schedule, scheduled(send_message_sched))]
struct SendMessageSchedule {
    // Mandatory fields:
    // ============================
    /// An identifier for the scheduled reducer call.
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,

    /// Information about when the reducer should be called.
    scheduled_at: ScheduleAt,

    // In addition to the mandatory fields, any number of fields can be added.
    // These can be used to provide extra information to the scheduled reducer.

    // Custom fields:
    // ============================
    /// The text of the scheduled message to send.
    text: String,
}

#[reducer]
fn send_message_sched(ctx: &ReducerContext, arg: SendMessageSchedule) -> Result<(), String> {
    let message_to_send = arg.text;

    log::info!("huh");

    _ = send_message(ctx, message_to_send);

    Ok(())
}

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    let ten_seconds = TimeDuration::from_micros(10_000_000);
    /* ctx.db.send_message_schedule().insert(SendMessageSchedule {
        scheduled_id: 0,
        text: "I'm a bot sending a message every 10 seconds".to_string(),

        // Creating a `ScheduleAt` from a `Duration` results in the reducer
        // being called in a loop, once every `loop_duration`.
        scheduled_at: ten_seconds.into(),
    }); */
}
