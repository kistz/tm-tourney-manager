mod way_point;

pub use way_point::WayPoint;

mod start_line;
pub use start_line::StartLine;

mod respawn;
pub use respawn::Respawn;

mod warm_up;
pub use warm_up::*;

mod give_up;
pub use give_up::GiveUp;

mod scores;
pub use scores::Scores;

/// Can hold every Event trasmitted trough the ModeScript events.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub enum Event {
    WayPoint(Box<WayPoint>),
    Respawn(Box<Respawn>),
    StartLine(Box<StartLine>),
    Scores(Box<Scores>),

    Custom(String),
}
