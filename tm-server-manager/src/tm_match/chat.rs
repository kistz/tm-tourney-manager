use spacetimedb::table;

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
}

impl MatchChat {
    pub(crate) fn new(match_id: u32, user_id: u32, message: String) -> Self {
        Self {
            message,
            user_id,
            match_id,
            id: 0,
        }
    }
}
