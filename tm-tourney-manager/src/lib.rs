use spacetimedb::{
    Identity, ReducerContext, ScheduleAt, SpacetimeType, Table, TimeDuration, Timestamp, reducer,
    table,
};
use tm_server_types::event::Event;

mod entity;
mod event;
mod r#match;
mod stage;
mod tournament;

#[table(name = event_template,public)]
pub struct EventTemplate {}

#[table(name = stage_template,public)]
pub struct StageTemplate {}

#[table(name = match_template,public)]
pub struct MatchTemplate {}

#[reducer]
pub fn post_event(ctx: &ReducerContext, event: Event) {
    /* log::info!("{event:?}");
    ctx.db.server_events().insert(ServerEvents {
        event: "jaaa".to_string(),
        content: "".to_owned(),
        typed: event,
    }); */
}

#[reducer(client_connected)]
// Called when a client connects to a SpacetimeDB database server
pub fn client_connected(ctx: &ReducerContext) {
    /* if let Some(user) = ctx.db.entity().identity().find(ctx.sender) {
        // If this is a returning user, i.e. we already have a `User` with this `Identity`,
        // set `online: true`, but leave `name` and `identity` unchanged.
        ctx.db.entity().identity().update(Entity {
            online: true,
            ..user
        });
    } else {
        // If this is a new user, create a `User` row for the `Identity`,
        // which is online, but hasn't set a name.
        /* ctx.db.user().insert(Entity {
            name: None,
            identity: ctx.sender,
            online: true,
        }); */
    } */
}

#[reducer(client_disconnected)]
// Called when a client disconnects from SpacetimeDB database server
pub fn identity_disconnected(ctx: &ReducerContext) {
    /* if let Some(user) = ctx.db.entity().identity().find(ctx.sender) {
        ctx.db.entity().identity().update(Entity {
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
    } */
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
    /* let message_to_send = arg.text;

    _ = send_message(ctx, message_to_send); */

    Ok(())
}

#[reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    let _ten_seconds = TimeDuration::from_micros(10_000_000);
    /* ctx.db.send_message_schedule().insert(SendMessageSchedule {
        scheduled_id: 0,
        text: "I'm a bot sending a message every 10 seconds".to_string(),

        // Creating a `ScheduleAt` from a `Duration` results in the reducer
        // being called in a loop, once every `loop_duration`.
        scheduled_at: ten_seconds.into(),
    }); */
}
