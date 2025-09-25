use spacetimedb::{Identity, SpacetimeType, table};

#[table(name = entity, public)]
pub struct Entity {
    #[primary_key]
    identity: Identity,
    name: String,
    online: bool,

    role: Roles,
}

#[derive(Debug, Clone, Copy, SpacetimeType)]
pub enum Roles {
    User,
    TrackmaniaServer,
}
