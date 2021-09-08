//
// Actual gaming
//

use std::sync::Arc;

use ron::de::from_str;
use serde::Deserialize;
use tui::style::Style;

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
        style: (
            logo:               (fg: Some(Red), bg: Some(Black), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            logs:               (bg: Some(Black), fg: Some(White), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            log_text:           (fg: Some(White), bg: None, add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            most_recent_log:    (bg: Some(White), fg: Some(Black), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            game_data:          (bg: Some(Black), fg: Some(White), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            split_name:         (fg: Some(White), bg: None, add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            split_value:        (fg: Some(Magenta), bg: None, add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            split_diff_pos:     (fg: Some(Green), bg: None, add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            split_diff_neg:     (fg: Some(Red), bg: None, add_modifier: (bits: 0), sub_modifier: (bits: 0))
        ),
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
            HomingSplits([
                (\"Levi\", 366.),
                // (\"490\", 490.),
                // (\"580\", 580.),
                (\"700\", 709.),
                (\"800\", 800.),
                (\"860\", 875.),
                (\"940\", 942.),
                (\"1000\", 996.),
                (\"1040\", 1047.),
                (\"1080\", 1091.),
                (\"1130\", 1133.),
                (\"1160\", 1163.),
            ]),
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
    pub style: Styles,
}

#[derive(Deserialize)]
pub struct Styles {
    pub logo: Style,
    pub logs: Style,
    pub log_text: Style,
    pub most_recent_log: Style,
    pub game_data: Style,
    pub split_name: Style,
    pub split_value: Style,
    pub split_diff_pos: Style,
    pub split_diff_neg: Style,
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
