//
// Actual gaming
//

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use ron::de::from_reader;
use ron::de::from_str;
use serde::Deserialize;
use tui::style::Style;

use crate::consts::LOGO_NEW;
use crate::ui::GameDataModules;

const DEFAULT_CFG: &str = "// ddstats-rust config

(
    offline: true,
    debug_logs: false,
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
        orb_connection_animation: true,
        style: (
            logo:               (fg: Some(Red), bg: Some(Black), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            logs:               (bg: Some(Black), fg: Some(White), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            log_text:           (fg: Some(White), bg: None, add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            most_recent_log:    (bg: Some(White), fg: Some(Black), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            game_data:          (bg: Some(Black), fg: Some(White), add_modifier: (bits: 0), sub_modifier: (bits: 0)),
            split_name:         (fg: Some(Yellow), bg: None, add_modifier: (bits: 0), sub_modifier: (bits: 0)),
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
            HomingUsed,
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
    #[serde(default)]
    pub logo: Logo,
    pub game_data_modules: Vec<GameDataModules>,
    pub style: Styles,
    pub orb_connection_animation: bool,
}

#[derive(Deserialize)]
pub struct Logo(pub String);

impl std::default::Default for Logo {
    fn default() -> Self {
        Logo(LOGO_NEW.to_string())
    }
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
    pub debug_logs: bool,
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

#[cfg(target_os = "linux")]
fn get_priority_file() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap();
    let config_path = exe_path.with_file_name("config.ron");
    if config_path.exists() {
        config_path
    } else {
        let home;
        if let Ok(xdg_home) = std::env::var("XDG_CONFIG_HOME") {
            home = xdg_home;
        } else {
            home = std::env::var("HOME").unwrap();
        }
        Path::new(format!("{}/.config/ddstats-rust/config.ron", home).as_str()).to_owned()
    }
}

#[cfg(target_os = "windows")]
fn get_priority_file() -> PathBuf {
    Path::new("./config.ron").to_owned()
}

fn get_config() -> DDStatsRustConfig {
    if get_priority_file().exists() {
        let f = File::open(&get_priority_file()).expect("Failed opening file");
        return from_reader(f).expect("Failed to load config");
    }

    if let Some(dir) = option_env!("CARGO_MANIFEST_DIR") {
        log::info!("Trying to load default cfg");
        let c = format!("{}/default_cfg.ron", dir);
        let fp = Path::new(c.as_str());
        if fp.exists() {
            let f = File::open(&fp).expect("Coudln't read default_cfg");
            return from_reader(f).expect("EE");
        }
    }

    from_str(DEFAULT_CFG).expect("FUN")
}
