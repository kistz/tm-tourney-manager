use spacetimedb::{ReducerContext, Uuid, reducer};
use tm_server_types::event::Event;

use crate::{
    authorization::Authorization,
    raw_server::{
        occupation::TabRawServerOccupationRead,
        player::{raw_server_player_add, raw_server_player_remove},
    },
    tm_match::{event::handle_match_event, tab_match},
    user::{UserRead, UserV1, UserWrite},
};

/// Servers call this to post the event stream.
#[reducer]
fn post_event(ctx: &ReducerContext, event: Event) -> Result<(), String> {
    let server_id = ctx.server_id()?;

    match &event {
        Event::PlayerConnect(player) => {
            let account_id = Uuid::parse_str(&player.account_id).unwrap();
            if !ctx.has_user(account_id) {
                let user = UserV1::new(account_id);
                _ = ctx.user_insert(user);
            }
            raw_server_player_add(ctx, account_id, player.is_spectator)?
        }
        Event::PlayerDisconnect(player) => {
            raw_server_player_remove(ctx, Uuid::parse_str(&player.account_id).unwrap())?
        }
        Event::PlayerInfoChanged(player) => {
            let spectator = player.spectator_status != 0;
            raw_server_player_add(ctx, Uuid::parse_str(&player.account_id).unwrap(), spectator)?
        }
        _ => (),
    }

    if let Some(node) = ctx.raw_server_occupation(server_id) {
        if node.is_match()
            && let Some(tm_match) = ctx.db.tab_match().id().find(node.split().1)
            && tm_match.is_live()
        {
            handle_match_event(ctx, tm_match.id, event)?
        }

        if node.is_server() {
            //TODO handle server events.
        }
    }
    Ok(())
}
