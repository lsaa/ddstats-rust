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
                username: other.block.username,
                survival_md5: other.block.survival_md5,
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
                per_enemy_alive_count: other.block.per_enemy_alive_count,
                per_enemy_kill_count: other.block.per_enemy_kill_count,
                time: other.block.time,
                is_player_alive: other.block.is_player_alive,
                is_in_game: other.block.is_in_game,
                is_time_attack_or_race_finished: other.block.is_time_attack_or_race_finished,
                replay_player_name: other.block.replay_player_name,
                status: other.block.status,
                stats_base: other.block.stats_base,
                stats_frames_loaded: other.block.stats_frames_loaded,
                stats_finished_loading: other.block.stats_finished_loading,
                starting_hand: other.block.starting_hand,
                starting_time: other.block.starting_time,
                starting_homing: other.block.starting_homing,
                prohibited_mods: other.block.prohibited_mods,
                game_mode: other.block.game_mode,
                replay_base: other.block.replay_base,
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
    pub username: [u8; 32],
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
    pub per_enemy_alive_count: [i16; 17],
    #[obake(cfg(">=5.0.0"))]
    pub per_enemy_kill_count: [i16; 17],
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
    pub replay_player_name: [u8; 32],
    #[obake(cfg(">=5.0.0"))]
    pub survival_md5: [u8; 16],
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
    pub stats_base: [u8; 8],
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
    pub replay_base: [u8; 8],
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
    pub per_enemy_alive_count: [i16; 17],
    #[obake(cfg(">=5.0.0"))]
    pub per_enemy_kill_count: [i16; 17],
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
            per_enemy_kill_count: other.per_enemy_kill_count,
            per_enemy_alive_count: other.per_enemy_alive_count,
            daggers_eaten: other.daggers_eaten,
            daggers_hit: other.daggers_hit,
            daggers_fired: other.daggers_fired,
            level_gems: other.level_gems,
        }
    }
}

