use chumsky::{
    Parser,
    combinator::DelimitedBy,
    primitive::{any, just},
    text,
};
use spacetimedb::ReducerContext;

use crate::tm_match::hook::MatchHookTriggerCtx;

/* pub(super) trait MessageParser {
    fn eval(&self, ctx: &ReducerContext, trigger_ctx: MatchHookTriggerCtx);
}

impl MessageParser for String {
    fn eval(&self, ctx: &ReducerContext, trigger_ctx: MatchHookTriggerCtx) {

    }
} */

enum Thing {
    Text(String),
    Var(String),
}

fn msg_eval<'a>() -> impl Parser<'a, &'a str, Vec<Thing>> {
    any().delimited_by(just("{{"), just("}}")).padded()
}
