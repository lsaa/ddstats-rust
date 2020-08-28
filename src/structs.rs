#[derive(Debug)]
pub struct GameData {
    pub player_id: i32,
    pub pb: f32,
    pub player_name: String,
    pub replay_player_name: String,
    pub replay_player_id: i32,

    //Game Data
    pub gems_upgrade: i32,
    pub homing: i32,
    pub is_dead: i32,

    //Game Stats
    pub timer: f32,
    pub gems_total: i32,
    pub daggers_fired: i32,
    pub daggers_hit: i32,
    pub enemies_alive: i32,
    pub enemies_killed: i32,
    pub is_replay: bool,
    pub is_alive: bool,
    pub death_type: i32,
    pub accuracy: f32,

    //Record Keeping
    pub level_2_time : f32,
    pub level_3_time: f32,
    pub level_4_time: f32,
    pub homing_max: i32,
    pub homing_max_time: f32,
}

impl GameData {
    pub fn reset(&mut self) {
        self.pb = 0.0;
        self.player_id = 0;
        self.replay_player_id = 0;
        self.player_name = String::new();
        self.replay_player_name = String::new();
        self.is_dead = 0;
        self.is_replay = false;
        self.is_alive = false;
        self.timer = 0.0;
        self.homing_max_time = 0.0;
        self.homing_max = 0;
        self.homing = 0;
        self.gems_total = 0;
        self.gems_upgrade = 0;
        self.level_2_time = 0.0;
        self.level_3_time = 0.0;
        self.level_4_time = 0.0;
        self.accuracy = 0.0;
        self.death_type = 0;
        self.enemies_alive = 0;
        self.enemies_killed = 0;
    }
}


pub enum STATE {
    NotConnected,
    Playing,
    Replay,
    Menu,
    Lobby,
    Dead,
}