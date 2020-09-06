use process_memory::DataMember;
//use crate::consts;
use std::sync::atomic::{AtomicBool, AtomicI32};
use serde_derive::{Serialize, Deserialize};

#[derive(Debug)]
pub struct GameData {
    pub last_fetch_data: Option<GameDataMembersRetrieval>,
    pub last_recording: f32,
    pub data_slices : DataSlices,

    //Record Keeping
    pub accuracy: f32,
    pub level_2_time : f32,
    pub level_3_time: f32,
    pub level_4_time: f32,
    pub homing_max: AtomicI32,
    pub homing_max_time: f32,
    pub enemies_alive_max: AtomicI32,
    pub enemies_alive_max_time: f32,
    pub enemies_alive_max_per_second: AtomicI32,
    pub homing_max_per_second: AtomicI32,
}

unsafe impl Send for GameData {}

impl GameData {
    pub fn new() -> GameData {
        return GameData {
            data_slices: DataSlices::new(),
            accuracy: 0.0,
            level_2_time: 0.0,
            level_3_time: 0.0,
            level_4_time: 0.0,
            homing_max: AtomicI32::new(0),
            homing_max_time: 0.0,
            last_recording: -1.0,
            homing_max_per_second: AtomicI32::new(0),
            enemies_alive_max_per_second: AtomicI32::new(0),
            last_fetch_data: None,
            enemies_alive_max: AtomicI32::new(0),
            enemies_alive_max_time: 0.0,
        }
    }

    pub fn log_run(&self) {
        //let death_type = format!("{:?}", self.last_fetch_data.as_ref().unwrap().death_type);
        //let death_type = death_type.parse::<i32>().unwrap();
        log::info!("\n\nRUN DUMP\n\n{:#?}", self);
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    NotConnected,
    Connecting,
    Playing,
    Replay,
    Menu,
    Lobby,
    Dead,
}

unsafe impl Send for State {}

#[derive(Debug)]
pub struct DataSlices {
    pub timer: Vec<f32>,
    pub total_gems: Vec<AtomicI32>,
    pub homing: Vec<AtomicI32>,
    pub daggers_fired: Vec<AtomicI32>,
    pub daggers_hit: Vec<AtomicI32>,
    pub enemies_killed: Vec<AtomicI32>,
    pub enemies_alive: Vec<AtomicI32>,
    pub granularity: AtomicI32,
}

unsafe impl Send for DataSlices {}

impl DataSlices {
    pub fn new() -> DataSlices {
        return DataSlices {
            timer: vec![],
            total_gems: vec![],
            homing: vec![],
            daggers_fired: vec![],
            daggers_hit: vec![],
            enemies_killed: vec![],
            enemies_alive: vec![],
            granularity: AtomicI32::new(1),
        };
    }
}

#[derive(Debug, Clone)]
pub struct GameDataMembers {
    pub player_id: DataMember<i32>,
    pub pb: DataMember<f32>,
    pub replay_player_id: DataMember<i32>,

    //Game Data
    pub gems_upgrade: DataMember<i32>,
    pub homing: DataMember<i32>,
    pub is_dead: DataMember<i32>,

    //Game Stats
    pub timer: DataMember<f32>,
    pub gems_total: DataMember<i32>,
    pub daggers_fired: DataMember<i32>,
    pub daggers_hit: DataMember<i32>,
    pub enemies_alive: DataMember<i32>,
    pub enemies_killed: DataMember<i32>,
    pub is_replay: DataMember<bool>,
    pub is_alive: DataMember<bool>,
    pub death_type: DataMember<i32>,
}

unsafe impl Send for GameDataMembers {}

#[derive(Debug)]
pub struct GameDataMembersRetrieval {
    pub player_id: AtomicI32,
    pub pb: f32,
    pub replay_player_id: AtomicI32,
    pub player_name: String,
    pub replay_player_name: String,

    //Game Data
    pub gems_upgrade: AtomicI32,
    pub homing: AtomicI32,
    pub is_dead: AtomicI32,

    //Game Stats
    pub timer: f32,
    pub gems_total: AtomicI32,
    pub daggers_fired: AtomicI32,
    pub daggers_hit: AtomicI32,
    pub enemies_alive: AtomicI32,
    pub enemies_killed: AtomicI32,
    pub is_replay: AtomicBool,
    pub is_alive: AtomicBool,
    pub death_type: AtomicI32,
}

unsafe impl Send for GameDataMembersRetrieval {}

#[derive(Serialize, Deserialize, Debug)]
pub struct MotdRespose {
    pub motd : String,
    pub valid_version: bool,
    pub update_available: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MotdRequest {
    pub version : String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitRunResponse {
    pub message: String,
    pub game_id: i32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameRecording {
    #[serde(rename(serialize = "playerID"))]
    pub player_id: i32,
    pub player_name: String,
    pub granularity: i32,
    pub in_game_timer: f32,
    pub in_game_timer_vector: Vec<f32>,
    pub gems: i32,
    pub gems_vector: Vec<i32>,
    pub level_two_time: f32,
    pub level_three_time: f32,
    pub level_four_time: f32,
    pub homing_daggers: i32,
    pub homing_daggers_vector: Vec<i32>,
    pub homing_daggers_max: i32,
    pub homing_daggers_max_time: f32,
    pub daggers_fired: i32,
    pub daggers_fired_vector: Vec<i32>,
    pub daggers_hit: i32,
    pub daggers_hit_vector: Vec<i32>,
    pub enemies_alive: i32,
    pub enemies_alive_vector: Vec<i32>,
    pub enemies_alive_max: i32,
    pub enemies_alive_max_time: f32,
    pub enemies_killed: i32,
    pub enemies_killed_vector: Vec<i32>,
    pub death_type: i32,
    #[serde(rename(serialize = "replayPlayerID"))]
    pub replay_player_id: i32,
    pub version: String,
    pub survival_hash: String,
}