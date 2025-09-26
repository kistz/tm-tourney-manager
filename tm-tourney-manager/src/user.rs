use spacetimedb::{Identity, SpacetimeType, table};

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: String,
    online: bool,
}

/* #[derive(Debug, Clone, Copy, SpacetimeType)]
pub enum Roles {
    User,
    TrackmaniaServer,
} */
