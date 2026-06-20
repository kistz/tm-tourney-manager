use spacetimedb::table;

use crate::tm_match::MatchStatus;

#[table(
    accessor= tab_match_chat,
    public
)]
pub struct MatchChat {
    message: String,

    #[index(hash)]
    user_id: u32,
    #[index(hash)]
    match_id: u32,
    #[auto_inc]
    #[primary_key]
    id: u32,
    #[default(MatchStatus::Live)]
    status: MatchStatus,
}

impl MatchChat {
    pub(crate) fn new(match_id: u32, status: MatchStatus, user_id: u32, message: String) -> Self {
        Self {
            message,
            user_id,
            match_id,
            id: 0,
            status,
        }
    }
}
