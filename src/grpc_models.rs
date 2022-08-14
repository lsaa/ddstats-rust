use ddcore_rs::models::StatsBlockWithFrames;

tonic::include_proto!("gamesubmission");

impl StatsFrame {
    pub fn from_game_frame(other: &ddcore_rs::models::StatsFrame) -> Self {
        Self {
            gems_collected: other.gems_collected,
            kills: other.kills,
            daggers_fired: other.daggers_fired,
            enemies_alive: other.enemies_alive,
            level_gems: other.level_gems,
            homing_daggers: other.homing,
            gems_despawned: other.gems_despawned,
            gems_eaten: other.gems_eaten,
            gems_total: other.gems_total,
            daggers_eaten: other.daggers_eaten,
            per_enemy_kill_count: other
                .per_enemy_kill_count
                .iter()
                .map(|x| *x as i32)
                .collect(),
            per_enemy_alive_count: other
                .per_enemy_alive_count
                .iter()
                .map(|x| *x as i32)
                .collect(),
            daggers_hit: other.daggers_hit,
        }
    }
}

impl SubmitGameRequest {
    pub fn from_compiled_run(other: crate::client::CompiledRun) -> Self {
        Self {
            version: other.version.clone(),
            player_id: other.player_id,
            player_name: other.player_name.clone(),
            level_hash_md5: other.level_hash_md5.clone(),
            time_lvl2: other.time_lvl2,
            time_lvl3: other.time_lvl3,
            time_lvl4: other.time_lvl4,
            time_levi_down: other.time_levi_down,
            time_orb_down: other.time_orb_down,
            enemies_alive_max: other.enemies_alive_max,
            enemies_alive_max_time: other.enemies_alive_max_time,
            homing_daggers_max: other.homing_daggers_max,
            homing_daggers_max_time: other.homing_daggers_max_time,
            death_type: other.death_type,
            is_replay: other.is_replay,
            replay_player_id: other.replay_player_id,
            daggers_hit: other.daggers_hit,
            daggers_fired: other.daggers_fired,
            enemies_alive: other.enemies_alive,
            enemies_killed: other.enemies_killed,
            gems_collected: other.gems_collected,
            gems_total: other.gems_total,
            homing_daggers: other.homing_daggers,
            level_gems: other.level_gems,
            time_max: other.time_max,
            daggers_eaten: other.daggers_eaten,
            gems_eaten: other.gems_eaten,
            gems_despawned: other.gems_despawned,
            per_enemy_alive_count: other
                .per_enemy_alive_count
                .iter()
                .map(|x| *x as i32)
                .collect(),
            per_enemy_kill_count: other
                .per_enemy_kill_count
                .iter()
                .map(|x| *x as i32)
                .collect(),
            frames: other
                .stats
                .iter()
                .map(StatsFrame::from_game_frame)
                .collect(),
        }
    }
}


#[obake::versioned]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct SavedData {
    #[obake(cfg(">=5.0.0"))]
    pub recorded_runs: u32,
    #[obake(inherit)]
    #[obake(cfg(">=5.0.0"))]
    pub recent_runs: RunList,
    #[obake(inherit)]
    #[obake(cfg(">=5.0.0"))]
    pub min: Run,
    #[obake(inherit)]
    #[obake(cfg(">=5.0.0"))]
    pub max: Run,
    #[obake(inherit)]
    #[obake(cfg(">=5.0.0"))]
    pub avg: Run,
}

impl SavedData {
    pub fn update_min(&mut self) {
        let new_run = self.recent_runs.runs.last().expect("this needs to be called after adding a new run");
        self.min.block.time_lvl2 = self.avg.block.time_lvl2.min(new_run.block.time_lvl2);
        self.min.block.time_lvl3 = self.avg.block.time_lvl3.min(new_run.block.time_lvl3);
        self.min.block.time_lvl4 = self.avg.block.time_lvl4.min(new_run.block.time_lvl4);
        self.min.block.time = self.avg.block.time.min(new_run.block.time);
        self.min.block.levi_down_time = self.avg.block.levi_down_time.min(new_run.block.levi_down_time);
        self.min.block.orb_down_time = self.avg.block.orb_down_time.min(new_run.block.orb_down_time);
        self.min.block.enemies_alive_max = self.avg.block.enemies_alive_max.min(new_run.block.enemies_alive_max);
        self.min.block.time_enemies_alive_max = self.avg.block.time_enemies_alive_max.min(new_run.block.time_enemies_alive_max);
        self.min.block.max_homing = self.avg.block.max_homing.min(new_run.block.max_homing);
        self.min.block.time_max_homing = self.avg.block.time_max_homing.min(new_run.block.time_max_homing);
        self.min.block.daggers_hit = self.avg.block.daggers_hit.min(new_run.block.daggers_hit);
        self.min.block.daggers_fired = self.avg.block.daggers_fired.min(new_run.block.daggers_fired);
        self.min.block.enemies_alive = self.avg.block.enemies_alive.min(new_run.block.enemies_alive);
        self.min.block.kills = self.avg.block.kills.min(new_run.block.kills);
        self.min.block.gems_collected = self.avg.block.gems_collected.min(new_run.block.gems_collected);
        self.min.block.gems_total = self.avg.block.gems_total.min(new_run.block.gems_total);
        self.min.block.homing = self.avg.block.homing.min(new_run.block.homing);
        self.min.block.time_max = self.avg.block.time_max.min(new_run.block.time_max);
        self.min.block.daggers_eaten = self.avg.block.daggers_eaten.min(new_run.block.daggers_eaten);
        self.min.block.gems_eaten = self.avg.block.gems_eaten.min(new_run.block.gems_eaten);
        self.min.block.gems_despawned = self.avg.block.gems_despawned.min(new_run.block.gems_despawned);
    }

    pub fn update_max(&mut self) {
        let new_run = self.recent_runs.runs.last().expect("this needs to be called after adding a new run");
        self.max.block.time_lvl2 = self.avg.block.time_lvl2.max(new_run.block.time_lvl2);
        self.max.block.time_lvl3 = self.avg.block.time_lvl3.max(new_run.block.time_lvl3);
        self.max.block.time_lvl4 = self.avg.block.time_lvl4.max(new_run.block.time_lvl4);
        self.max.block.time = self.avg.block.time.max(new_run.block.time);
        self.max.block.levi_down_time = self.avg.block.levi_down_time.max(new_run.block.levi_down_time);
        self.max.block.orb_down_time = self.avg.block.orb_down_time.max(new_run.block.orb_down_time);
        self.max.block.enemies_alive_max = self.avg.block.enemies_alive_max.max(new_run.block.enemies_alive_max);
        self.max.block.time_enemies_alive_max = self.avg.block.time_enemies_alive_max.max(new_run.block.time_enemies_alive_max);
        self.max.block.max_homing = self.avg.block.max_homing.max(new_run.block.max_homing);
        self.max.block.time_max_homing = self.avg.block.time_max_homing.max(new_run.block.time_max_homing);
        self.max.block.daggers_hit = self.avg.block.daggers_hit.max(new_run.block.daggers_hit);
        self.max.block.daggers_fired = self.avg.block.daggers_fired.max(new_run.block.daggers_fired);
        self.max.block.enemies_alive = self.avg.block.enemies_alive.max(new_run.block.enemies_alive);
        self.max.block.kills = self.avg.block.kills.max(new_run.block.kills);
        self.max.block.gems_collected = self.avg.block.gems_collected.max(new_run.block.gems_collected);
        self.max.block.gems_total = self.avg.block.gems_total.max(new_run.block.gems_total);
        self.max.block.homing = self.avg.block.homing.max(new_run.block.homing);
        self.max.block.time_max = self.avg.block.time_max.max(new_run.block.time_max);
        self.max.block.daggers_eaten = self.avg.block.daggers_eaten.max(new_run.block.daggers_eaten);
        self.max.block.gems_eaten = self.avg.block.gems_eaten.max(new_run.block.gems_eaten);
        self.max.block.gems_despawned = self.avg.block.gems_despawned.max(new_run.block.gems_despawned);
    }

    pub fn update_avg(&mut self) {
        let new_run = self.recent_runs.runs.last().expect("this needs to be called after adding a new run");
        self.avg.block.time_lvl2 = self.avg.block.time_lvl2 + ((new_run.block.time_lvl2 - self.avg.block.time_lvl2) / self.recorded_runs as f32);
        self.avg.block.time_lvl3 = self.avg.block.time_lvl3 + ((new_run.block.time_lvl3 - self.avg.block.time_lvl3) / self.recorded_runs as f32);
        self.avg.block.time_lvl4 = self.avg.block.time_lvl4 + ((new_run.block.time_lvl4 - self.avg.block.time_lvl4) / self.recorded_runs as f32);
        self.avg.block.time = self.avg.block.time + ((new_run.block.time - self.avg.block.time) / self.recorded_runs as f32);
        self.avg.block.levi_down_time = self.avg.block.levi_down_time + ((new_run.block.levi_down_time - self.avg.block.levi_down_time) / self.recorded_runs as f32);
        self.avg.block.orb_down_time = self.avg.block.orb_down_time + ((new_run.block.orb_down_time - self.avg.block.orb_down_time) / self.recorded_runs as f32);
        self.avg.block.enemies_alive_max = self.avg.block.enemies_alive_max + ((new_run.block.enemies_alive_max - self.avg.block.enemies_alive_max) / self.recorded_runs as i32);
        self.avg.block.time_enemies_alive_max = self.avg.block.time_enemies_alive_max + ((new_run.block.time_enemies_alive_max - self.avg.block.time_enemies_alive_max) / self.recorded_runs as f32);
        self.avg.block.max_homing = self.avg.block.max_homing + ((new_run.block.max_homing - self.avg.block.max_homing) / self.recorded_runs as i32);
        self.avg.block.time_max_homing = self.avg.block.time_max_homing + ((new_run.block.time_max_homing - self.avg.block.time_max_homing) / self.recorded_runs as f32);
        self.avg.block.daggers_hit = self.avg.block.daggers_hit + ((new_run.block.daggers_hit - self.avg.block.daggers_hit) / self.recorded_runs as i32);
        self.avg.block.daggers_fired = self.avg.block.daggers_fired + ((new_run.block.daggers_fired - self.avg.block.daggers_fired) / self.recorded_runs as i32);
        self.avg.block.enemies_alive = self.avg.block.enemies_alive + ((new_run.block.enemies_alive - self.avg.block.enemies_alive) / self.recorded_runs as i32);
        self.avg.block.kills = self.avg.block.kills + ((new_run.block.kills - self.avg.block.kills) / self.recorded_runs as i32);
        self.avg.block.gems_collected = self.avg.block.gems_collected + ((new_run.block.gems_collected - self.avg.block.gems_collected) / self.recorded_runs as i32);
        self.avg.block.gems_total = self.avg.block.gems_total + ((new_run.block.gems_total - self.avg.block.gems_total) / self.recorded_runs as i32);
        self.avg.block.homing = self.avg.block.homing + ((new_run.block.homing - self.avg.block.homing) / self.recorded_runs as i32);
        self.avg.block.time_max = self.avg.block.time_max + ((new_run.block.time_max - self.avg.block.time_max) / self.recorded_runs as f32);
        self.avg.block.daggers_eaten = self.avg.block.daggers_eaten + ((new_run.block.daggers_eaten - self.avg.block.daggers_eaten) / self.recorded_runs as i32);
        self.avg.block.gems_eaten = self.avg.block.gems_eaten + ((new_run.block.gems_eaten - self.avg.block.gems_eaten) / self.recorded_runs as i32);
        self.avg.block.gems_despawned = self.avg.block.gems_despawned + ((new_run.block.gems_despawned - self.avg.block.gems_despawned) / self.recorded_runs as i32);
    }
}


#[obake::versioned]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Run {
    #[obake(inherit)]
    #[obake(cfg(">=5.0.0"))]
    pub block: Block,
    #[obake(inherit)] 
    #[obake(cfg(">=5.0.0"))]
    pub frames: Frames,
}

impl Run {
    pub fn from_sbwf(other: &StatsBlockWithFrames) -> Self {
        Self {
            block: Block {
                ddstats_version: other.block.ddstats_version,
                player_id: other.block.player_id,
                username: other.block.username.to_vec(),
                survival_md5: other.block.survival_md5.to_vec(),
                time_lvl2: other.block.time_lvl2,
                time_lvl3: other.block.time_lvl3,
                time_lvl4: other.block.time_lvl4,
                levi_down_time: other.block.levi_down_time,
                orb_down_time: other.block.orb_down_time,
                enemies_alive_max: other.block.enemies_alive_max,
                time_enemies_alive_max: other.block.time_enemies_alive_max,
                max_homing: other.block.max_homing,
                time_max_homing: other.block.time_max_homing,
                death_type: other.block.death_type,
                is_replay: other.block.is_replay,
                replay_player_id: other.block.replay_player_id,
                daggers_hit: other.block.daggers_hit,
                daggers_fired: other.block.daggers_fired,
                enemies_alive: other.block.enemies_alive,
                kills: other.block.kills,
                gems_collected: other.block.gems_collected,
                gems_total: other.block.gems_total,
                homing: other.block.homing,
                level_gems: other.block.level_gems,
                time_max: other.block.time_max,
                daggers_eaten: other.block.daggers_eaten,
                gems_eaten: other.block.gems_eaten,
                gems_despawned: other.block.gems_despawned,
                per_enemy_alive_count: other.block.per_enemy_alive_count.to_vec(),
                per_enemy_kill_count: other.block.per_enemy_kill_count.to_vec(),
                time: other.block.time,
                is_player_alive: other.block.is_player_alive,
                is_in_game: other.block.is_in_game,
                is_time_attack_or_race_finished: other.block.is_time_attack_or_race_finished,
                replay_player_name: other.block.replay_player_name.to_vec(),
                status: other.block.status,
                stats_base: other.block.stats_base.to_vec(),
                stats_frames_loaded: other.block.stats_frames_loaded,
                stats_finished_loading: other.block.stats_finished_loading,
                starting_hand: other.block.starting_hand,
                starting_time: other.block.starting_time,
                starting_homing: other.block.starting_homing,
                prohibited_mods: other.block.prohibited_mods,
                game_mode: other.block.game_mode,
                replay_base: other.block.replay_base.to_vec(),
                replay_flag: other.block.replay_flag,
                replay_buffer_length: other.block.replay_buffer_length,
            },
            frames: Frames { frames: other.frames.iter().map(Frame::from_data_frame).collect() },
        }
    }
}

#[obake::versioned]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct RunList {
    #[obake(cfg(">=5.0.0"))]
    pub runs: Vec<Run>,
}

#[obake::versioned]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Block {
    #[obake(cfg(">=5.0.0"))]
    pub ddstats_version: i32,
    #[obake(cfg(">=5.0.0"))]
    pub player_id: i32,
    #[obake(cfg(">=5.0.0"))]
    pub username: Vec<u8>,
    #[obake(cfg(">=5.0.0"))]
    pub time: f32,
    #[obake(cfg(">=5.0.0"))]
    pub gems_collected: i32,
    #[obake(cfg(">=5.0.0"))]
    pub kills: i32,
    #[obake(cfg(">=5.0.0"))]
    pub daggers_fired: i32,
    #[obake(cfg(">=5.0.0"))]
    pub daggers_hit: i32,
    #[obake(cfg(">=5.0.0"))]
    pub enemies_alive: i32,
    #[obake(cfg(">=5.0.0"))]
    pub level_gems: i32,
    #[obake(cfg(">=5.0.0"))]
    pub homing: i32,
    #[obake(cfg(">=5.0.0"))]
    pub gems_despawned: i32,
    #[obake(cfg(">=5.0.0"))]
    pub gems_eaten: i32,
    #[obake(cfg(">=5.0.0"))]
    pub gems_total: i32,
    #[obake(cfg(">=5.0.0"))]
    pub daggers_eaten: i32,
    #[obake(cfg(">=5.0.0"))]
    pub per_enemy_alive_count: Vec<i16>,
    #[obake(cfg(">=5.0.0"))]
    pub per_enemy_kill_count: Vec<i16>,
    #[obake(cfg(">=5.0.0"))]
    pub is_player_alive: bool,
    #[obake(cfg(">=5.0.0"))]
    pub is_replay: bool,
    #[obake(cfg(">=5.0.0"))]
    pub death_type: u8,
    #[obake(cfg(">=5.0.0"))]
    pub is_in_game: bool,
    #[obake(cfg(">=5.0.0"))]
    pub replay_player_id: i32,
    #[obake(cfg(">=5.0.0"))]
    pub replay_player_name: Vec<u8>,
    #[obake(cfg(">=5.0.0"))]
    pub survival_md5: Vec<u8>,
    #[obake(cfg(">=5.0.0"))]
    pub time_lvl2: f32,
    #[obake(cfg(">=5.0.0"))]
    pub time_lvl3: f32,
    #[obake(cfg(">=5.0.0"))]
    pub time_lvl4: f32,
    #[obake(cfg(">=5.0.0"))]
    pub levi_down_time: f32,
    #[obake(cfg(">=5.0.0"))]
    pub orb_down_time: f32,
    #[obake(cfg(">=5.0.0"))]
    pub status: i32,
    #[obake(cfg(">=5.0.0"))]
    pub max_homing: i32,
    #[obake(cfg(">=5.0.0"))]
    pub time_max_homing: f32,
    #[obake(cfg(">=5.0.0"))]
    pub enemies_alive_max: i32,
    #[obake(cfg(">=5.0.0"))]
    pub time_enemies_alive_max: f32,
    #[obake(cfg(">=5.0.0"))]
    pub time_max: f32,
    #[obake(cfg(">=5.0.0"))]
    pub stats_base: Vec<u8>,
    #[obake(cfg(">=5.0.0"))]
    pub stats_frames_loaded: i32,
    #[obake(cfg(">=5.0.0"))]
    pub stats_finished_loading: bool,
    #[obake(cfg(">=5.0.0"))]
    pub starting_hand: i32,
    #[obake(cfg(">=5.0.0"))]
    pub starting_homing: i32,
    #[obake(cfg(">=5.0.0"))]
    pub starting_time: f32,
    #[obake(cfg(">=5.0.0"))]
    pub prohibited_mods: bool,
    #[obake(cfg(">=5.0.0"))]
    pub replay_base: Vec<u8>,
    #[obake(cfg(">=5.0.0"))]
    pub replay_buffer_length: i32,
    #[obake(cfg(">=5.0.0"))]
    pub replay_flag: bool,
    #[obake(cfg(">=5.0.0"))]
    pub game_mode: u8,
    #[obake(cfg(">=5.0.0"))]
    pub is_time_attack_or_race_finished: bool,
}

#[obake::versioned]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Frames {
    pub frames: Vec<Frame>,
}

#[obake::versioned]
#[obake(version("5.0.0"))]
#[obake(derive(serde::Serialize, serde::Deserialize))]
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Frame {
    #[obake(cfg(">=5.0.0"))]
    pub gems_collected: i32,
    #[obake(cfg(">=5.0.0"))]
    pub kills: i32,
    #[obake(cfg(">=5.0.0"))]
    pub daggers_fired: i32,
    #[obake(cfg(">=5.0.0"))]
    pub daggers_hit: i32,
    #[obake(cfg(">=5.0.0"))]
    pub enemies_alive: i32,
    #[obake(cfg(">=5.0.0"))]
    pub level_gems: i32,
    #[obake(cfg(">=5.0.0"))]
    pub homing: i32,
    #[obake(cfg(">=5.0.0"))]
    pub gems_despawned: i32,
    #[obake(cfg(">=5.0.0"))]
    pub gems_eaten: i32,
    #[obake(cfg(">=5.0.0"))]
    pub gems_total: i32,
    #[obake(cfg(">=5.0.0"))]
    pub daggers_eaten: i32,
    #[obake(cfg(">=5.0.0"))]
    pub per_enemy_alive_count: Vec<i16>,
    #[obake(cfg(">=5.0.0"))]
    pub per_enemy_kill_count: Vec<i16>,
}

impl Frame {
    pub fn from_data_frame(other: &ddcore_rs::models::StatsFrame) -> Self {
        Self {
            gems_total: other.gems_total,
            gems_eaten: other.gems_eaten,
            gems_despawned: other.gems_despawned,
            gems_collected: other.gems_collected,
            homing: other.homing,
            enemies_alive: other.enemies_alive,
            kills: other.kills,
            per_enemy_kill_count: other.per_enemy_kill_count.to_vec(),
            per_enemy_alive_count: other.per_enemy_alive_count.to_vec(),
            daggers_eaten: other.daggers_eaten,
            daggers_hit: other.daggers_hit,
            daggers_fired: other.daggers_fired,
            level_gems: other.level_gems,
        }
    }
}

