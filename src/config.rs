use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct DDStatsRustConfig {
    pub host: String,
    pub offline: bool,
    pub auto_clipboard: bool,
    pub stream: Stream,
    pub submit: Submit,
    pub discord: Discord
}

#[derive(Serialize, Deserialize)]
pub struct Stream {
    pub stats: bool,
    pub replay_stats: bool,
    pub non_default_spawnsets: bool
}

#[derive(Serialize, Deserialize)]
pub struct Submit {
    pub stats: bool,
    pub replay_stats: bool,
    pub non_default_spawnsets: bool
}

#[derive(Serialize, Deserialize)]
pub struct Discord {
    pub notify_above_1000: bool,
    pub notify_player_best: bool,
    pub notify_custom_spawnsets: bool
}

impl ::std::default::Default for DDStatsRustConfig {
    fn default() -> Self { 
        Self { 
            offline: true, 
            auto_clipboard: true,
            host: String::from("https://ddstats.com"),
            submit: Submit {
                stats: true,
                replay_stats: true,
                non_default_spawnsets: true,
            },
            stream: Stream {
                stats: true,
                replay_stats: true,
                non_default_spawnsets: true,
            },
            discord: Discord {
                notify_above_1000: true,
                notify_player_best: true,
                notify_custom_spawnsets: false,
            },
        } 
    }
}

pub fn get_config() -> Result<DDStatsRustConfig, confy::ConfyError> {
    let cfg = confy::load_path("./config.toml")?;
    Ok(cfg)
}