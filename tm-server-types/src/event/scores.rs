use crate::{
    base::{Player, Team},
    event::Event,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct Scores {
    #[serde(rename = "responseid")]
    response_id: String,
    section: String,
    #[serde(rename = "useteams")]
    use_teams: bool,

    #[serde(rename = "winnerteam")]
    winner_team: i32,
    #[serde(rename = "winnerplayer")]
    winner_player: String,

    teams: Vec<Team>,
    players: Vec<Player>,
}

impl<'a> From<&'a Event> for &'a Scores {
    fn from(value: &'a Event) -> Self {
        match value {
            Event::Scores(event) => event,
            _ => panic!("Wrong argument for this"),
        }
    }
}
