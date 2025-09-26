mod player;
pub use player::Player;

mod team;
pub use team::Team;

mod map;
pub use map::Map;

mod time;
pub use time::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct UbisoftId {
    account_id: String,
}
