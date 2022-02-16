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
