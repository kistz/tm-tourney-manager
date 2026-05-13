use spacetimedb::{SpacetimeType, table};
use tm_server_types::event::PlayerConnect;

use crate::tm_match::{MatchV1, state::MatchState};

/* #[table(
    accessor=tab_match_hook,
    index(accessor=match_trigger,hash(columns=[match_id,trigger]))
    public
)] */
struct MatchHook {
    //#[index(hash)]
    match_id: u32,
    trigger: MatchHookTrigger,

    action: MatchHookAction,
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
    ChatSend(MatchHookActionChatSendCtx),
    //ChatSendToPlayer()
}

#[derive(Debug, SpacetimeType)]
struct MatchHookActionChatSendCtx {}

// MatchHookCapture <- could be global state or metadata you can optionally insert.
