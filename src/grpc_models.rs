tonic::include_proto!("gamesubmission");

impl StatsFrame {
    pub fn from_game_frame(other: &crate::mem::StatsFrame) -> Self {
        Self {
            gems_collected: other.gems_collected.clone(),
            kills: other.kills.clone(),
            daggers_fired: other.daggers_fired.clone(),
            enemies_alive: other.enemies_alive.clone(),
            level_gems: other.level_gems.clone(),
            homing_daggers: other.homing.clone(),
            gems_despawned: other.gems_despawned.clone(),
            gems_eaten: other.gems_eaten.clone(),
            gems_total: other.gems_total.clone(),
            daggers_eaten: other.daggers_eaten.clone(),
            per_enemy_kill_count: other
                .per_enemy_kill_count
                .iter()
                .map(|x| x.clone() as i32)
                .collect(),
            per_enemy_alive_count: other
                .per_enemy_alive_count
                .iter()
                .map(|x| x.clone() as i32)
                .collect(),
            daggers_hit: other.daggers_hit.clone(),
        }
    }
}

impl SubmitGameRequest {
    pub fn from_compiled_run(other: crate::client::CompiledRun) -> Self {
        let death_type = if other.death_type == 16 {
            11
        } else {
            other.death_type
        };
        Self {
            version: other.version.clone(),
            player_id: other.player_id.clone(),
            player_name: other.player_name.clone(),
            level_hash_md5: other.level_hash_md5.clone(),
            time_lvl2: other.time_lvl2.clone(),
            time_lvl3: other.time_lvl3.clone(),
            time_lvl4: other.time_lvl4.clone(),
            time_levi_down: other.time_levi_down.clone(),
            time_orb_down: other.time_orb_down.clone(),
            enemies_alive_max: other.enemies_alive_max.clone(),
            enemies_alive_max_time: other.enemies_alive_max_time.clone(),
            homing_daggers_max: other.homing_daggers_max.clone(),
            homing_daggers_max_time: other.homing_daggers_max_time.clone(),
            death_type,
            is_replay: other.is_replay.clone(),
            replay_player_id: other.replay_player_id.clone(),
            daggers_hit: other.daggers_hit.clone(),
            daggers_fired: other.daggers_fired.clone(),
            enemies_alive: other.enemies_alive.clone(),
            enemies_killed: other.enemies_killed.clone(),
            gems_collected: other.gems_collected.clone(),
            gems_total: other.gems_total.clone(),
            homing_daggers: other.homing_daggers.clone(),
            level_gems: other.level_gems.clone(),
            time_max: other.time_max.clone(),
            daggers_eaten: other.daggers_eaten.clone(),
            gems_eaten: other.gems_eaten.clone(),
            gems_despawned: other.gems_despawned.clone(),
            per_enemy_alive_count: other
                .per_enemy_alive_count
                .iter()
                .map(|x| x.clone() as i32)
                .collect(),
            per_enemy_kill_count: other
                .per_enemy_kill_count
                .iter()
                .map(|x| x.clone() as i32)
                .collect(),
            frames: other
                .stats
                .iter()
                .map(|st| StatsFrame::from_game_frame(st))
                .collect(),
        }
    }
}
