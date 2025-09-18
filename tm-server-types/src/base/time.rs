/// The time which a player took to finish.
/// Dnfs or no time yet is represented by none.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
#[cfg_attr(feature = "serde", serde(from = "i32"))]
pub enum RoundTime {
    None,
    Time(u32),
}

#[cfg(feature = "serde")]
impl From<i32> for RoundTime {
    fn from(value: i32) -> Self {
        match value {
            -1 => RoundTime::None,
            _ => RoundTime::Time(value as u32),
        }
    }
}
