use process_memory::DataMember;
use crate::consts;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize};
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
            last_fetch_data: None
        }
    }

    pub fn log_run(&self) {
        let death_type = format!("{:?}", self.last_fetch_data.as_ref().unwrap().death_type);
        let death_type = death_type.parse::<i32>().unwrap();

        log::info!("
        {:?}
        TIME {:.4}s
        ACCURACY {:.2}
        L2 {:?}s
        L3 {:?}s
        L4 {:?}s
        HOMING MAX {:?} - {:?}s
        ENEMIES ALIVE {:?}
        ENEMIES KILLED {:?}", 
        consts::DEATH_TYPES[death_type as usize],
        self.last_fetch_data.as_ref().unwrap().timer, 
        self.accuracy, self.level_2_time, self.level_3_time, 
        self.level_4_time, self.homing_max, self.homing_max_time,
        self.last_fetch_data.as_ref().unwrap().enemies_alive,
        self.last_fetch_data.as_ref().unwrap().enemies_killed);
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
