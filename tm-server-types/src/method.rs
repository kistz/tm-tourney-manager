#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub enum Method {
    /// ===============
    /// XML-RPC Methods
    /// ===============
    ///
    ListMethods,

    /// ===============
    /// ModeScript Methods
    /// ===============
    ///
    GetMethodsList,
}
