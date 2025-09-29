use spacetimedb::{ReducerContext, reducer};

pub mod event;
pub mod leaderboard;
pub mod r#match;
pub mod server;
pub mod stage;
pub mod tournament;
pub mod user;

#[reducer(init)]
pub fn init(_ctx: &ReducerContext) {
    /* let _ten_seconds = TimeDuration::from_micros(10_000_000);
    /* ctx.db.send_message_schedule().insert(SendMessageSchedule {
        scheduled_id: 0,
        text: "I'm a bot sending a message every 10 seconds".to_string(),

        // Creating a `ScheduleAt` from a `Duration` results in the reducer
        // being called in a loop, once every `loop_duration`.
        scheduled_at: ten_seconds.into(),
    }); */ */
}

#[reducer(client_connected)]
// Called when a client connects to a SpacetimeDB database server
pub fn client_connected(_ctx: &ReducerContext) {
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
pub fn identity_disconnected(_ctx: &ReducerContext) {
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
