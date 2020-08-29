use crate::structs::GameData;

#[cfg(target_os = "linux")]
use sysinfo::{ProcessExt, System, SystemExt};
use crate::consts::{LINUX_GAME_ADDRESS, LINUX_GAME_STATS_ADDRESS};

#[cfg(target_os = "windows")]
use winapi;

pub fn try_read_std_string_utf8(handle: process_memory::ProcessHandle, starting_offsets: Vec<usize>) -> Result<String, std::io::Error> {
    use process_memory::*;
    let offset = DataMember::<u8>::new_offset(handle, starting_offsets);
    let len_off = DataMember::<i32>::new_offset(handle, vec![offset.get_offset()? - 8]);
    let len = len_off.read()? as usize;
    let mut bytes = vec![0_u8; len];
    handle.copy_address(offset.get_offset()?, &mut bytes)?;
    String::from_utf8(bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

#[cfg(target_os = "linux")]
pub fn get_pid(process_name: &str) -> process_memory::Pid {
    let s = System::new_all();
    for process in s.get_process_by_name(process_name) {
        return process.pid();
    }
    return 0;
}

#[cfg(target_os = "linux")]
pub fn fetch_stats(handle: process_memory::ProcessHandle) -> Result<GameData, std::io::Error> {
    use process_memory::*;
    //let timer = DataMember::<f32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x214]); //Engine timer
    let pb = DataMember::<f32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x3AC]); 
    let timer = DataMember::<f32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x224]);
    let gems_total = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x244]);
    let enemies_killed = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x240]);
    let enemies_alive = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x280]);
    let daggers_fired = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x238]);
    let daggers_hit = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x23C]);
    let is_replay = DataMember::<bool>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x41D]);
    let death_type = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x248]);
    let is_alive = DataMember::<bool>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x228]);    
    let player_id = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0xB0]);    
    let player_name = try_read_std_string_utf8(handle, vec![LINUX_GAME_STATS_ADDRESS,  0xC8]);
    let replay_player_name = try_read_std_string_utf8(handle, vec![LINUX_GAME_STATS_ADDRESS,  0x430]);

    let gems = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_ADDRESS, 0, 0x2DC]);
    let homing = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_ADDRESS, 0, 0x2E8]);
    let is_dead = DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_ADDRESS, 0, 0xE4]); 

    let data = GameData {
        timer: timer.read()?,
        pb: pb.read()?,
        gems_total: gems_total.read()?,
        gems_upgrade: gems.read()?,
        is_dead: is_dead.read()?,
        daggers_fired: daggers_fired.read()?,
        daggers_hit: daggers_hit.read()?,
        accuracy: daggers_hit.read()? as f32 / daggers_fired.read()? as f32,
        player_id: player_id.read()?,
        player_name: player_name?,
        replay_player_id: 1,
        replay_player_name: replay_player_name?,
        is_alive: is_alive.read()?,
        is_replay: is_replay.read()?,
        homing: homing.read()?,
        homing_max: 11031,
        homing_max_time: 0.0,
        death_type: death_type.read()?,
        enemies_alive: enemies_alive.read()?,
        enemies_killed: enemies_killed.read()?,
        level_2_time: 0.0,
        level_3_time: 0.0,
        level_4_time: 0.0,
    };

    return Ok(data);
}

#[cfg(target_os = "windows")]
pub fn get_pid(process_name: &str) {
    
}