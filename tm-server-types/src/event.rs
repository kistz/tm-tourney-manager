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

mod map;
pub use map::*;

mod round;
pub use round::*;

mod turn;
pub use turn::*;

mod podium;
pub use podium::*;

mod play_loop;
pub use play_loop::*;

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

    LoadingMapStart(LoadingMapStart),
    LoadingMapEnd(LoadingMapEnd),
    StartMapStart(StartMap),
    StartMapEnd(StartMap),
    EndMapStart(EndMapStart),
    EndMapEnd(EndMapEnd),
    UnloadingMapStart(UnloadingMapStart),
    UnloadingMapEnd(UnloadingMapEnd),

    StartTurnStart(StartTurn),
    StartTurnEnd(StartTurn),

    PlayLoopStart(PlayLoopStart),
    PlayLoopEnd(PlayLoopEnd),

    EndRoundStart(EndRoundStart),
    EndRoundEnd(EndRoundEnd),

    PodiumStart(Podium),
    PodiumEnd(Podium),

    Custom(Custom),
}

impl Event {
    pub fn new(name: String, body: String) -> Self {
        //TODO include event names
        match name.as_str() {
            "Trackmania.Event.WayPoint" => Event::WayPoint(json::from_str(&body).unwrap()),
            "Trackmania.Event.Respawn" => Event::Respawn(json::from_str(&body).unwrap()),
            "Trackmania.Scores" => Event::Scores(json::from_str(&body).unwrap()),
            "Trackmania.Event.StartLine" => Event::StartLine(json::from_str(&body).unwrap()),

            "Maniaplanet.LoadingMap_Start" => {
                Event::LoadingMapStart(json::from_str(&body).unwrap())
            }
            "Maniaplanet.LoadingMap_End" => Event::LoadingMapEnd(json::from_str(&body).unwrap()),
            "Maniaplanet.StartMap_Start" => Event::StartMapStart(json::from_str(&body).unwrap()),
            "Maniaplanet.StartMap_End" => Event::StartMapEnd(json::from_str(&body).unwrap()),
            "Maniaplanet.EndMap_Start" => Event::EndMapStart(json::from_str(&body).unwrap()),
            "Maniaplanet.EndMap_End" => Event::EndMapEnd(json::from_str(&body).unwrap()),
            "Maniaplanet.UnloadingMap_Start" => {
                Event::UnloadingMapStart(json::from_str(&body).unwrap())
            }
            "Maniaplanet.UnloadingMap_End" => {
                Event::UnloadingMapEnd(json::from_str(&body).unwrap())
            }

            "Maniaplanet.StartTurn_Start" => Event::StartTurnStart(json::from_str(&body).unwrap()),
            "Maniaplanet.StartTurn_End" => Event::StartTurnEnd(json::from_str(&body).unwrap()),

            "Maniaplanet.StartPlayLoop" => Event::PlayLoopStart(json::from_str(&body).unwrap()),
            "Maniaplanet.EndPlayLoop" => Event::PlayLoopEnd(json::from_str(&body).unwrap()),

            "Maniaplanet.EndRound_Start" => Event::EndRoundStart(json::from_str(&body).unwrap()),
            "Maniaplanet.EndRound_End" => Event::EndRoundEnd(json::from_str(&body).unwrap()),

            "Maniaplanet.Podium_Start" => Event::PodiumStart(json::from_str(&body).unwrap()),
            "Maniaplanet.Podium_End" => Event::PodiumEnd(json::from_str(&body).unwrap()),

            _ => Event::Custom(Custom::new(name, body)),
        }
    }
}
