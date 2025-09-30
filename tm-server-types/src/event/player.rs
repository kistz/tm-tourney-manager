use crate::base::UbisoftId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct PlayerChat {
    //TODO
    /*  #[cfg_attr(feature = "serde", serde(rename = "Login", deserialize_with = ""))]
    account_id: UbisoftId, */
    #[cfg_attr(feature = "serde", serde(rename = "Login"))]
    login: UbisoftId,
    #[cfg_attr(feature = "serde", serde(rename = "Text"))]
    text: String,
    #[cfg_attr(feature = "serde", serde(rename = "IsRegisteredCmd"))]
    is_registered_cmd: bool,
    #[cfg_attr(feature = "serde", serde(rename = "Options"))]
    options: i32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct PlayerConnect {
    //TODO
    /*  #[cfg_attr(feature = "serde", serde(rename = "Login", deserialize_with = ""))]
    account_id: UbisoftId, */
    #[cfg_attr(feature = "serde", serde(rename = "Login"))]
    login: UbisoftId,
    #[cfg_attr(feature = "serde", serde(rename = "IsSpectator"))]
    is_spectator: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "spacetime", derive(spacetimedb_lib::SpacetimeType))]
#[cfg_attr(feature = "spacetime", sats(crate = spacetimedb_lib))]
pub struct PlayerDisconnect {
    //TODO
    /*  #[cfg_attr(feature = "serde", serde(rename = "Login", deserialize_with = ""))]
    account_id: UbisoftId, */
    #[cfg_attr(feature = "serde", serde(rename = "Login"))]
    login: UbisoftId,
    #[cfg_attr(feature = "serde", serde(rename = "DisconnectReason"))]
    disconnect_reason: String,
}
