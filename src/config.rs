//
// Actual gaming
//

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;
use arc_swap::Guard;
use lazy_static::lazy_static;
use ron::de::from_reader;
use ron::de::from_str;
use serde::Deserialize;
use tui::style::Color;
use tui::style::Style;

use crate::consts::LOGO_NEW;
use crate::threads::AAS;
use crate::ui::GameDataModules;

const DEFAULT_CFG: &str = include_str!("../default_cfg.ron");

#[derive(Deserialize, serde::Serialize)]
pub struct UiConf {
    pub enabled: bool,
    pub logo_style: LogoStyle,
    pub hide_logs: bool,
    #[serde(default)]
    pub logo: Logo,
    pub game_data_modules: Vec<GameDataModules>,
    pub style: Styles,
    pub orb_connection_animation: bool,
    pub column_distance: u16,
}

#[derive(Deserialize, serde::Serialize)]
pub struct Logo(pub String);

#[derive(Deserialize, PartialEq, serde::Serialize)]
pub enum LogoStyle {
    Auto,
    Mini,
    Full,
    Off,
}

impl std::default::Default for Logo {
    fn default() -> Self {
        Logo(LOGO_NEW.to_string())
    }
}

#[derive(Deserialize, Clone, serde::Serialize)]
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

impl std::default::Default for Styles {
    fn default() -> Self {
        Self {
            logo: Style::default().fg(Color::Red),
            logs: Style::default().fg(Color::White),
            log_text: Style::default().fg(Color::White),
            most_recent_log: Style::default().bg(Color::White).fg(Color::Black),
            game_data: Style::default().fg(Color::White),
            split_name: Style::default().fg(Color::Yellow),
            split_value: Style::default().fg(Color::Magenta),
            split_diff_pos: Style::default().fg(Color::Green),
            split_diff_neg: Style::default().fg(Color::Red),
        }
    }
}

#[derive(Deserialize, serde::Serialize)]
pub struct DDStatsRustConfig {
    pub host: String,
    pub grpc_host: String,
    pub offline: bool,
    pub debug_logs: bool,
    pub auto_clipboard: bool,
    pub stream: Stream,
    pub submit: Submit,
    pub discord: Discord,
    pub ui_conf: UiConf,
    pub linux_restart_as_child: bool,
    pub use_linux_proton: bool,
    pub ddcl: Ddcl,
    #[serde(default)]
    pub tray_icon: bool,
    #[serde(default)]
    pub hide_window_on_start: bool,
    #[serde(default)]
    pub upload_replays_automatically: bool,
    #[serde(default)]
    pub block_marker_override: Option<usize>,
    #[serde(default)]
    pub process_name_override: Option<String>,
    pub open_game_on_replay_request: bool,
}

#[derive(Deserialize, serde::Serialize)]
pub struct Stream {
    pub stats: bool,
    pub replay_stats: bool,
    pub non_default_spawnsets: bool,
}

#[derive(Deserialize, serde::Serialize)]
pub struct Submit {
    pub stats: bool,
    pub replay_stats: bool,
    pub non_default_spawnsets: bool,
}

#[derive(Deserialize, serde::Serialize)]
pub struct Discord {
    pub notify_above_1000: bool,
    pub notify_player_best: bool,
    pub notify_custom_spawnsets: bool,
}

#[derive(Deserialize, serde::Serialize)]
pub struct Ddcl {
    pub submit: bool,
    pub replays: bool,
}

impl std::default::Default for Ddcl {
    fn default() -> Self {
        Self {
            submit: true,
            replays: true,
        }
    }
}

lazy_static! {
    pub static ref CONFIG: AAS<DDStatsRustConfig> = Arc::new(ArcSwap::from_pointee(get_config()));
}

#[cfg(target_os = "linux")]
fn get_priority_file() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap();
    let config_path = exe_path.with_file_name("config.ron");
    if config_path.exists() {
        config_path
    } else {
        let mut home;
        if let Ok(xdg_home) = std::env::var("XDG_CONFIG_HOME") {
            home = xdg_home;
        } else {
            home = std::env::var("HOME").unwrap();
            home.push_str("/.config");
        }
        Path::new(format!("{}/ddstats-rust/config.ron", home).as_str()).to_owned()
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

    if let Some(default_cfg) = default_cfg_locate() {
        if default_cfg.exists() {
            let mut f = File::open(&default_cfg).expect("Can't read default config");
            if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
                let c = format!("{}/ddstats-rust/", config_home.as_str());
                let cpath = Path::new(c.as_str());
                if std::fs::create_dir_all(cpath).is_ok() {
                    let c = format!("{}/ddstats-rust/config.ron", config_home.as_str());
                    let cpath = Path::new(c.as_str());
                    let mut f_new = File::create(cpath).expect("Coudln't create config");
                    std::io::copy(&mut f, &mut f_new).expect("Couldn't write to config file");
                }
            }

            return from_reader(f).expect("couldn't read default config");
        }
    }

    from_str(DEFAULT_CFG).expect("FUN")
}

#[cfg(target_os = "windows")]
fn default_cfg_locate() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "linux")]
fn default_cfg_locate() -> Option<PathBuf> {
    Some(PathBuf::from("/usr/share/doc/ddstats-rust/default_cfg.ron"))
}

pub fn cfg<'a>() -> Guard<Arc<DDStatsRustConfig>> {
    CONFIG.load()
}
