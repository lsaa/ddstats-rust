//
// Client Module for devildaggers.info api
//

use futures::StreamExt;
use hyper::http::Request;
use hyper::{Body, Client, Method};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use std::sync::Arc;
use std::time::Duration;

pub const DDCL_MIMIC: &str = "1.3.0.0";

lazy_static! {
    pub static ref DD_MEMORY_MARKER: Arc<usize> = Arc::new(get_marker());
    pub static ref DDLC_UP_TO_DATE: Arc<bool> = Arc::new(calc_ddcl());
}

#[cfg(target_os = "linux")]
fn get_marker() -> usize {
    use crate::consts;
    let r = futures::executor::block_on(get_ddstats_memory_marker(OperatingSystem::Linux));
    if r.is_err() {
        return consts::LINUX_BLOCK_START;
    }
    r.unwrap().value
}

#[cfg(target_os = "windows")]
fn get_marker() -> usize {
    use crate::consts;
    let r = futures::executor::block_on(get_ddstats_memory_marker(OperatingSystem::Windows));
    if r.is_err() {
        return consts::WINDOWS_BLOCK_START;
    }
    r.unwrap().value
}

#[allow(unreachable_code)]
fn calc_ddcl() -> bool {
    return true;
    futures::executor::block_on(is_ddcl_up_to_date())
}

#[derive(Debug, serde::Serialize)]
pub enum OperatingSystem {
    Windows,
    Linux,
}

#[derive(serde::Deserialize, Debug)]
pub struct MarkerResponse {
    pub value: usize,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub display_name: String,
    pub version_number: String,
    pub version_number_required: String,
    pub changelog: Vec<ChangelogEntry>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangelogEntry {
    pub version_number: String,
    pub date: String,
    pub changes: Vec<Change>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub description: String,
    pub sub_changes: Option<Vec<String>>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRunRequest {
    pub survival_hash_md5: String,
    pub player_id: i32,
    pub player_name: String,
    pub time: i32,
    pub gems_collected: i32,
    pub enemies_killed: i32,
    pub daggers_fired: i32,
    pub daggers_hit: i32,
    pub enemies_alive: i32,
    pub homing_daggers: i32,
    pub homing_daggers_eaten: i32,
    pub gems_despawned: i32,
    pub gems_eaten: i32,
    pub gems_total: i32,
    pub death_type: u8,
    pub level_up_time2: i32,
    pub level_up_time3: i32,
    pub level_up_time4: i32,
    pub client_version: String,
    pub operating_system: OperatingSystem,
    pub build_mode: String,
    pub validation: String,
    pub is_replay: bool,
    pub prohibited_mods: bool,
    pub game_states: Vec<GameState>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub gems_collected: i32,
    pub enemies_killed: i32,
    pub daggers_fired: i32,
    pub daggers_hit: i32,
    pub enemies_alive: i32,
    pub homing_daggers: i32,
    pub homing_daggers_eaten: i32,
    pub gems_despawned: i32,
    pub gems_eaten: i32,
    pub gems_total: i32,
    pub skull1s_alive: i32,
    pub skull2s_alive: i32,
    pub skull3s_alive: i32,
    pub spiderlings_alive: i32,
    pub skull4s_alive: i32,
    pub squid1s_alive: i32,
    pub squid2s_alive: i32,
    pub squid3s_alive: i32,
    pub centipedes_alive: i32,
    pub gigapedes_alive: i32,
    pub spider1s_alive: i32,
    pub spider2s_alive: i32,
    pub leviathans_alive: i32,
    pub orbs_alive: i32,
    pub thorns_alive: i32,
    pub ghostpedes_alive: i32,
    pub skull1s_killed: i32,
    pub skull2s_killed: i32,
    pub skull3s_killed: i32,
    pub spiderlings_killed: i32,
    pub skull4s_killed: i32,
    pub squid1s_killed: i32,
    pub squid2s_killed: i32,
    pub squid3s_killed: i32,
    pub centipedes_killed: i32,
    pub gigapedes_killed: i32,
    pub spider1s_killed: i32,
    pub spider2s_killed: i32,
    pub leviathans_killed: i32,
    pub orbs_killed: i32,
    pub thorns_killed: i32,
    pub ghostpedes_killed: i32,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn get_ddstats_memory_marker(os: OperatingSystem) -> Result<MarkerResponse> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let path = format!("api/process-memory/marker?operatingSystem={:?}", os);
    let uri = format!("https://devildaggers.info/{}", path);
    let req = Request::builder()
        .header("accept", "application/json")
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let mut res = tokio::time::timeout(Duration::from_secs(4), client.request(req)).await??;
    log::info!("Pulled Marker Sucessfully");
    let mut body = Vec::new();
    while let Some(chunk) = res.body_mut().next().await {
        body.extend_from_slice(&chunk?);
    }
    let res: MarkerResponse = serde_json::from_slice(&body)?;
    Ok(res)
}

pub async fn get_ddcl() -> Result<Tool> {
    get_tool("DevilDaggersCustomLeaderboards".into()).await
}

#[rustfmt::skip]
pub async fn is_ddcl_up_to_date() -> bool {
    let mut ddcl = get_ddcl().await;
    let cfg = crate::config::cfg();
    if !cfg.ddcl.submit && !cfg.ddcl.replays {
        return true;
    }

    for _i in 0..5 {
        if ddcl.is_ok() { break; }
        ddcl = get_ddcl().await;
    }

    if ddcl.is_err() { return false; }
    let ddcl = ddcl.unwrap();

    let vs: Vec<u16> = ddcl.version_number_required.split(".").map(|z| str::parse::<u16>(z).unwrap()).collect();
    let vs2: Vec<u16> = DDCL_MIMIC.split(".").map(|z| str::parse::<u16>(z).unwrap()).collect();
    let m = if vs.len() > vs2.len() { vs2.len() } else { vs.len() };

    for i in 0..m {
        if vs.iter().nth(i).unwrap() > vs2.iter().nth(i).unwrap() {
            return false
        }
    }

    true
}

pub async fn get_tool(tool_name: String) -> Result<Tool> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let path = format!("api/tools/{}", tool_name);
    let uri = format!("https://devildaggers.info/{}", path);
    let req = Request::builder()
        .header("accept", "application/json")
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let mut res = client.request(req).await?;
    let mut body = Vec::new();
    while let Some(chunk) = res.body_mut().next().await {
        body.extend_from_slice(&chunk?);
    }
    let res: Tool = serde_json::from_slice(&body)?;
    Ok(res)
}
