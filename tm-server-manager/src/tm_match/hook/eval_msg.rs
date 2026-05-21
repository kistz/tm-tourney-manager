/* use chumsky::{
    IterParser, Parser,
    combinator::DelimitedBy,
    primitive::{any, just},
    recursive::recursive,
    text,
};

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

pub(super) fn msg_eval<'a>() -> impl Parser<'a, &'a str, Vec<String>> {
    any()
        .repeated()
        .collect::<String>()
        .delimited_by(just("{{"), just("}}"))
        .or(any().repeated().collect::<String>())
        .repeated()
        .collect::<Vec<_>>()

    /* let ident = any::<_, extra::Err<Simple<char>>>()
        .filter(|c: &char| c.is_alphabetic())
        .repeated()
        .at_least(1)
        .collect::<String>();

    let num = text::int(10).from_str().unwrapped();

    let s_expr = recursive(|s_expr| {
        s_expr
            //.collect::<Vec<_>>()
            .map(Thing::Text)
            .delimited_by(just('('), just(')'))
    }); */
}

/* #[test]
fn test_eval() {
    let ja = msg_eval().parse("This is a good {{server}} server message {{wowie}}");
} */
 */
