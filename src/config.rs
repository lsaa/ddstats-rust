//
// Actual gaming
//

use std::sync::Arc;

use ron::de::from_str;
use serde::Deserialize;

use crate::ui::GameDataModules;

const DEFAULT_CFG: &str = "// ddstats-rust config

(
    offline: true,
    host: \"https://ddstats.com\",
    auto_clipboard: true,
    stream: (
        stats: false,
        replay_stats: false,
        non_default_spawnsets: false,
    ),
    submit: (
        stats: false,
        replay_stats: false,
        non_default_spawnsets: false,
    ),
    discord: (
        notify_above_1000: true,
        notify_player_best: true,
        notify_custom_spawnsets: false,
    ),

    // UI Config
    ui_conf: (
        hide_logo: false,
        hide_logs: false,
        game_data_modules: [
            RunData,
            Timer,
            Gems,
            Homing(true),
            Kills,
            Accuracy,
            GemsLost(true),
            CollectionAccuracy,
            Spacing,
            HomingSplits([366., 709., 800., 875., 942., 996., 1047., 1091., 1133.])
        ],
        logo: \"

████████▄  ████████▄     ▄████████     ███        ▄████████     ███        ▄████████
███   ▀███ ███   ▀███   ███    ███ ▀█████████▄   ███    ███ ▀█████████▄   ███    ███
███    ███ ███    ███   ███    █▀     ▀███▀▀██   ███    ███    ▀███▀▀██   ███    █▀ㅤ
███    ███ ███    ███   ███            ███   ▀   ███    ███     ███   ▀   ███ㅤㅤㅤㅤ
███    ███ ███    ███ ▀███████████     ███     ▀███████████     ███     ▀███████████
███    ███ ███    ███          ███     ███       ███    ███     ███              ███
███   ▄███ ███   ▄███    ▄█    ███     ███       ███    ███     ███        ▄█    ███
████████▀  ████████▀   ▄████████▀     ▄████▀     ███    █▀     ▄████▀    ▄████████▀ㅤ
v0.6.8                                                                          rust\",
    ),

)";

#[derive(Deserialize)]
pub struct UiConf {
    pub hide_logo: bool,
    pub hide_logs: bool,
    pub logo: String,
    pub game_data_modules: Vec<GameDataModules>,
}

#[derive(Deserialize)]
pub struct DDStatsRustConfig {
    pub host: String,
    pub offline: bool,
    pub auto_clipboard: bool,
    pub stream: Stream,
    pub submit: Submit,
    pub discord: Discord,
    pub ui_conf: UiConf,
}

#[derive(Deserialize)]
pub struct Stream {
    pub stats: bool,
    pub replay_stats: bool,
    pub non_default_spawnsets: bool,
}

#[derive(Deserialize)]
pub struct Submit {
    pub stats: bool,
    pub replay_stats: bool,
    pub non_default_spawnsets: bool,
}

#[derive(Deserialize)]
pub struct Discord {
    pub notify_above_1000: bool,
    pub notify_player_best: bool,
    pub notify_custom_spawnsets: bool,
}

thread_local! {
    pub static CONFIG: Arc<DDStatsRustConfig> = Arc::new(get_config());
}

fn get_config() -> DDStatsRustConfig {
    from_str(DEFAULT_CFG).expect("FUN")
}
