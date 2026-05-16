use spacetimedb::{ReducerContext, SpacetimeType, table};
use tm_server_types::event::{Event, PlayerConnect};

use crate::tm_match::{MatchV1, hook::eval_msg::MessageParser, state::MatchState};

mod eval_msg;

#[table(
    accessor=tab_match_hook,
    index(accessor=match_trigger,hash(columns=[match_id,trigger])),
    public
)]
struct MatchHook {
    //#[index(hash)]
    match_id: u32,
    trigger: MatchHookTrigger,

    actions: Vec<MatchHookAction>,
}

#[derive(Debug, SpacetimeType)]
enum MatchHookTrigger {
    PlayerConnected,
}

enum GlobalCtx {
    MatchMetadata(MatchV1),
    MatchState(MatchState),
}

#[derive(Debug)]
enum MatchHookTriggerCtx<'a> {
    PlayerConnected(&'a PlayerConnect),
}

#[derive(Debug, SpacetimeType)]
enum MatchHookAction {
    ChatSend(String),
    ChatSendToPlayer(MatchHookActionChatSendToPlayerCtx),
}

impl MatchHookAction {
    fn execute(self, ctx: &ReducerContext, trigger_ctx: MatchHookTriggerCtx) {
        match self {
            MatchHookAction::ChatSend(msg) => msg.eval(ctx, trigger_ctx),
            MatchHookAction::ChatSendToPlayer(match_hook_action_chat_send_to_player_ctx) => todo!(),
        }
    }
}

#[derive(Debug, SpacetimeType)]
struct MatchHookActionChatSendToPlayerCtx {
    player: u32,
    message: String,
}

// MatchHookCapture <- could be global state or metadata you can optionally insert.

pub(crate) fn match_handle_hook(
    ctx: &ReducerContext,
    mut state: MatchState,
    event: &Event,
) -> Result<(), String> {
    match event {
        Event::PlayerConnect(event) => {
            let event_hooks = ctx
                .db
                .tab_match_hook()
                .match_trigger()
                .filter((state.match_id, MatchHookTrigger::PlayerConnected));

            for hook in event_hooks {
                for action in hook.actions {
                    let event_ctx = MatchHookTriggerCtx::PlayerConnected(event);
                    action.execute(ctx, event_ctx);
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
