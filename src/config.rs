//
// Actual gaming
//

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use arc_swap::ArcSwap;
use arc_swap::Guard;
use lazy_static::lazy_static;
use ron::de::from_reader;
use ron::de::from_str;
use ron::ser::PrettyConfig;
use serde::Deserialize;
use tui::style::Style;
use crate::grpc_models::SavedData;
use crate::threads::AAS;
use crate::ui::modules::GameDataModules;

const DEFAULT_CFG: &str = include_str!("../default_cfg.ron");
const DEFAULT_SAVED: &str = include_str!("../default_data.ron");

type VersionedCfg = <DDStatsRustConfig as obake::Versioned>::Versioned;
type VersionedData = <SavedData as obake::Versioned>::Versioned;

lazy_static! {
    pub static ref CONFIG: AAS<DDStatsRustConfig> = Arc::new(ArcSwap::from_pointee(get_config()));
    pub static ref SAVED_DATA: AAS<SavedData> = Arc::new(ArcSwap::from_pointee(get_saved_data()));
}

#[obake::versioned]
#[obake(version("1.0.0"))]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(Deserialize, serde::Serialize, Clone)]
pub struct DDStatsRustConfig {
    #[obake(cfg(">=1.0.0"))]
    pub host: String,
    #[obake(cfg(">=1.0.0"))]
    pub grpc_host: String,
    #[obake(cfg(">=1.0.0"))]
    pub offline: bool,
    #[obake(cfg(">=1.0.0"))]
    pub debug_logs: bool,
    #[obake(cfg(">=1.0.0"))]
    pub auto_clipboard: bool,
    #[obake(cfg(">=1.0.0"))]
    pub linux_restart_as_child: bool,
    #[obake(cfg(">=1.0.0"))]
    pub use_linux_proton: bool,
    #[obake(cfg(">=1.0.0"))]
    pub tray_icon: bool,
    #[obake(cfg(">=1.0.0"))]
    pub hide_window_on_start: bool,
    #[obake(cfg(">=1.0.0"))]
    pub upload_replays_automatically: bool,
    #[obake(cfg(">=1.0.0"))]
    pub block_marker_override: Option<usize>,
    #[obake(cfg(">=1.0.0"))]
    pub process_name_override: Option<String>,
    #[obake(cfg(">=1.0.0"))]
    pub open_game_on_replay_request: bool,
    #[obake(cfg(">=5.0.0"))]
    pub saved_games_max: u32,
    #[obake(cfg(">=5.0.0"))]
    pub record_threshold: f32,

    #[obake(cfg(">=1.0.0"))]
    #[obake(inherit)]
    pub stream: Stream,
    #[obake(cfg(">=1.0.0"))]
    #[obake(inherit)]
    pub submit: Submit,
    #[obake(cfg(">=1.0.0"))]
    #[obake(inherit)]
    pub discord: Discord,
    #[obake(cfg(">=1.0.0"))]
    #[obake(inherit)]
    pub ui_conf: UiConf,
}



#[obake::versioned]
#[obake(version("1.0.0"))]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(Deserialize, serde::Serialize, Clone)]
pub struct UiConf {
    #[obake(cfg(">=1.0.0"))]
    pub enabled: bool,
    #[obake(cfg(">=1.0.0"))]
    pub logo_style: LogoStyle,
    #[obake(cfg(">=1.0.0"))]
    pub hide_logs: bool,
    #[obake(cfg(">=1.0.0"))]
    pub logo: Option<String>,
    #[obake(cfg(">=1.0.0"))]
    pub orb_connection_animation: bool,
    #[obake(cfg(">=1.0.0"))]
    pub column_distance: u16,
    #[obake(cfg(">=1.0.0"))]
    pub show_help_on_border: bool,
    #[obake(cfg(">=1.0.0"))]
    pub current_split_marker: String,
    #[obake(cfg(">=1.0.0"))]
    pub current_split_live_change: bool,
    #[obake(cfg(">=1.0.0"))]
    pub always_show_splits: bool,

    // keeping this down here to hope that the deserializer leaves the junk in the bottom
    #[obake(cfg(">=1.0.0"))]
    pub game_data_modules: Vec<GameDataModules>,
    #[obake(cfg(">=1.0.0"))]
    #[obake(inherit)]
    pub theming: Theming,
}

#[derive(Deserialize, PartialEq, serde::Serialize, Clone)]
pub enum LogoStyle {
    Auto,
    Mini,
    Full,
    Off,
}

#[obake::versioned]
#[obake(version("1.0.0"))]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(Deserialize, Clone, serde::Serialize)]
pub struct Theming {
    #[obake(inherit)]
    #[obake(cfg(">=1.0.0"))]
    pub styles: Styles,
}

#[obake::versioned]
#[obake(version("1.0.0"))]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(Deserialize, Clone, serde::Serialize)]
pub struct Styles {
    #[obake(cfg(">=1.0.0"))]
    pub text: Style,
    #[obake(cfg(">=1.0.0"))]
    pub logo: Style,
    #[obake(cfg(">=1.0.0"))]
    pub logs: Style,
    #[obake(cfg(">=1.0.0"))]
    pub logs_title: Style,
    #[obake(cfg(">=1.0.0"))]
    pub log_text: Style,
    #[obake(cfg(">=1.0.0"))]
    pub most_recent_log: Style,
    #[obake(cfg(">=1.0.0"))]
    pub game_data: Style,
    #[obake(cfg(">=1.0.0"))]
    pub game_data_title: Style,
    #[obake(cfg(">=1.0.0"))]
    pub split_name: Style,
    #[obake(cfg(">=1.0.0"))]
    pub accent: Style,
    #[obake(cfg(">=1.0.0"))]
    pub split_diff_pos: Style,
    #[obake(cfg(">=1.0.0"))]
    pub split_diff_neg: Style,
    #[obake(cfg(">=1.0.0"))]
    pub split_diff_neutral: Style,
    #[obake(cfg(">=1.0.0"))]
    pub split_diff_gold: Style,
}

#[obake::versioned]
#[obake(version("1.0.0"))]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(Deserialize, serde::Serialize, Clone)]
pub struct Stream {
    #[obake(cfg(">=1.0.0"))]
    pub stats: bool,
    #[obake(cfg(">=1.0.0"))]
    pub replay_stats: bool,
    #[obake(cfg(">=1.0.0"))]
    pub non_default_spawnsets: bool,
}

#[obake::versioned]
#[obake(version("1.0.0"))]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(Deserialize, serde::Serialize, Clone)]
pub struct Submit {
    #[obake(cfg(">=1.0.0"))]
    pub stats: bool,
    #[obake(cfg(">=1.0.0"))]
    pub replay_stats: bool,
    #[obake(cfg(">=1.0.0"))]
    pub non_default_spawnsets: bool,
    #[obake(cfg(">=1.0.0"))]
    pub ddcl: bool,
}

#[obake::versioned]
#[obake(version("1.0.0"))]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(Deserialize, serde::Serialize, Clone)]
pub struct Discord {
    #[obake(cfg(">=1.0.0"))]
    pub notify_above_1000: bool,
    #[obake(cfg(">=1.0.0"))]
    pub notify_player_best: bool,
    #[obake(cfg(">=1.0.0"))]
    pub notify_custom_spawnsets: bool,
}

// VERSIONS
include!("versioning/v5.rs");

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
    let exe_path = std::env::current_exe().unwrap();
    exe_path.with_file_name("config.ron")
}

fn try_priority_file() -> anyhow::Result<VersionedCfg> {
    if get_priority_file().exists() {
        let f = File::open(&get_priority_file())?;
        return Ok(from_reader(f)?);
    }
    
    anyhow::bail!("Priority config file not found");
}

fn try_dbg_file() -> anyhow::Result<VersionedCfg> {
    if let Some(dir) = option_env!("CARGO_MANIFEST_DIR") {
        log::info!("Trying to load default cfg");
        let c = format!("{}/default_cfg.ron", dir);
        let fp = Path::new(c.as_str());
        if fp.exists() {
            let f = File::open(&fp)?;
            return Ok(from_reader(f)?);
        }
    }
    
    anyhow::bail!("Debug config not found");
}

pub fn try_save_with_backup() -> anyhow::Result<()> {
    let current_cfg = (*CONFIG.load_full()).clone();
    let current_cfg: VersionedCfg = current_cfg.into();
    let serialized = ron::ser::to_string_pretty(
        &current_cfg, 
        PrettyConfig::new().indentor("    ".to_string()).depth_limit(4).decimal_floats(true)
    )?;

    // Bail if any of the steps fail
    // 1 - Find best config file

    let best_file = get_priority_file();

    if !best_file.exists() {
        anyhow::bail!("No config file found.");
    }

    log::info!("Found best file: {best_file:?}");

    // 2 - Create backup

    let mut current_config_file = File::open(&best_file)?;
    let backup_path = best_file.with_file_name("config.backup");
    log::info!("backup file path: {backup_path:?}");
    let mut backup_file = File::create(backup_path.to_str().ok_or_else(|| anyhow::anyhow!("Failed to transform backup path to str"))?)?;
    std::io::copy(&mut current_config_file, &mut backup_file)?;
    log::info!("Created backup");

    // 3 - Write to config file

    let mut current_config_file = File::create(&best_file)?;
    let mut serialized_file_reader = BufReader::new(serialized.as_bytes());
    std::io::copy(&mut serialized_file_reader, &mut current_config_file)?;
    log::info!("Wrote to cfg");

    // 4 - Cleanup backup

    std::fs::remove_file(backup_path)?;
    log::info!("Cleaned backup");

    Ok(())
}

fn try_create_config_file() -> anyhow::Result<()> {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        let c = format!("{}/ddstats-rust/", config_home.as_str());
        let cpath = Path::new(c.as_str());
        if std::fs::create_dir_all(cpath).is_ok() {
            let c = format!("{}/ddstats-rust/config.ron", config_home.as_str());
            let cpath = Path::new(c.as_str());
            let mut f_new = File::create(cpath)?;
            std::io::copy(&mut BufReader::new(DEFAULT_CFG.as_bytes()), &mut f_new)?;
        }
    } else if Path::new("~/.config").exists() {
        let config_home = "~/.config".to_string();
        let c = format!("{}/ddstats-rust/", config_home.as_str());
        let cpath = Path::new(c.as_str());
        if std::fs::create_dir_all(cpath).is_ok() {
            let c = format!("{}/ddstats-rust/config.ron", config_home.as_str());
            let cpath = Path::new(c.as_str());
            let mut f_new = File::create(cpath)?;
            std::io::copy(&mut BufReader::new(DEFAULT_CFG.as_bytes()), &mut f_new)?;
        }
    } else {
        let cpath = Path::new("./config.ron");
        let mut f_new = File::create(cpath)?;
        std::io::copy(&mut BufReader::new(DEFAULT_CFG.as_bytes()), &mut f_new)?;
    }

    Ok(())
}

fn get_config() -> DDStatsRustConfig {
    if let Ok(conf) = try_priority_file() {
        return conf.into();
    }

    if let Ok(conf) = try_dbg_file() {
        return conf.into();
    }

    if let Err(e) = try_create_config_file() {
        log::warn!("Failed to create config file: {e:?}");
    }

    // Try to read from config file inside executable as last resort
    let cf: VersionedCfg = from_str(DEFAULT_CFG).unwrap();
    cf.into()
}

fn get_saved_data() -> SavedData {
    let cf: VersionedData = from_str(DEFAULT_SAVED).unwrap();
    cf.into()
}

pub fn get_log_file_path() -> PathBuf {
    if Path::new("./config.ron").to_owned().exists() {
        Path::new("./debug_logs.txt").to_owned()
    } else if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        let c = format!("{}/ddstats-rust/debug_logs.txt", config_home.as_str());
        Path::new(&c).to_owned()
    } else if Path::new("~/.config/ddstats-rust").exists() {
        Path::new("~/.config/ddstats-rust/debug_logs.txt").to_owned()
    } else {
        Path::new("./debug_logs.txt").to_owned()
    }
}

pub fn cfg() -> Guard<Arc<DDStatsRustConfig>> {
    CONFIG.load()
}


/*
        let a: <DDStatsRustConfig as obake::Versioned>::Versioned =(DDStatsRustConfig {
            host: "c".to_string(),
            grpc_host: "a".to_string(),
            offline: false,
            ui_conf: UiConf { 
                enabled: true, 
                logo_style: LogoStyle::Auto, 
                hide_logs:false, 
                logo: None, 
                game_data_modules: vec![], 
                style: Styles { 
                    text: Style::default(),
                    logo: Style::default(), 
                    logs: Style::default(), 
                    logs_title: Style::default(), 
                    log_text: Style::default(), 
                    most_recent_log: Style::default(), 
                    game_data: Style::default(), 
                    game_data_title: Style::default(), 
                    split_name: Style::default(), 
                    accent: Style::default(), 
                    split_diff_pos: Style::default(), 
                    split_diff_neg: Style::default(), 
                    split_diff_neutral: Style::default(), 
                    split_diff_gold: Style::default() 
                }, 
                orb_connection_animation: false, 
                column_distance: 20 
            },
            use_linux_proton: false,
            upload_replays_automatically: true,
            auto_clipboard: true,
            tray_icon: false,
            hide_window_on_start: false,
            open_game_on_replay_request: true,
            process_name_override: None,
            block_marker_override: None,
            debug_logs: true,
            discord: Discord { notify_above_1000: true, notify_player_best: true, notify_custom_spawnsets: false },
            stream: Stream { stats: true, replay_stats: true, non_default_spawnsets: false },
            submit: Submit { stats: true, replay_stats: true, non_default_spawnsets: false, ddcl: true },
            linux_restart_as_child: false,
        }).into();

        println!("{:#}", ron::ser::to_string(&a).expect("a"));
        */


