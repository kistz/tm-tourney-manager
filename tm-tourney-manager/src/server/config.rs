use spacetimedb::table;
use tm_server_types::config::ServerConfig;

#[table(name=tm_server_config, public)]
pub struct TmServerConfig {
    #[auto_inc]
    #[primary_key]
    pub id: u64,

    /// Ubi id of the creator
    creator: String,

    config: ServerConfig,
}

impl TmServerConfig {
    pub fn get_config(self) -> ServerConfig {
        self.config
    }
}
