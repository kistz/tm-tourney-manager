use crate::event::Event;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct KnockoutElimination {
    #[cfg_attr(feature = "serde", serde(rename = "accountids"))]
    pub account_ids: Vec<String>,
}

impl<'a> From<&'a Event> for &'a KnockoutElimination {
    #[inline]
    fn from(value: &'a Event) -> Self {
        match value {
            Event::KnockoutElimination(event) => event,
            _ => unreachable!(),
        }
    }
}
