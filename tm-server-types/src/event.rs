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

mod custom;
pub use custom::Custom;

/// Can hold every Event trasmitted trough the ModeScript events.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub enum Event {
    WayPoint(WayPoint),
    Respawn(Respawn),
    StartLine(StartLine),
    Scores(Scores),
    GiveUp(GiveUp),

    Custom(Custom),
}

impl Event {
    pub fn new(name: String, body: String) -> Self {
        //TODO include event name
        match name.as_str() {
            "Trackmania.Event.WayPoint" => Event::WayPoint(json::from_str(&body).unwrap()),
            _ => Event::Custom(Custom::new(name, body)),
        }
    }
}
