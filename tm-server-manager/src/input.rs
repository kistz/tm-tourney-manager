use spacetimedb::table;

#[table(accessor= tab_input)]
pub struct InputV1 {
    name: String,

    #[auto_inc]
    #[primary_key]
    pub(crate) id: u32,

    #[index(hash)]
    parent_id: u32,
    //TODO do we need this?
    //status:
    template: bool,
}
