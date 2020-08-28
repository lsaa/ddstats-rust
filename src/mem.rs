use std::{thread, time};
use crate::structs::GameData;

#[cfg(target_os = "linux")]
use sysinfo::{ProcessExt, System, SystemExt};

#[cfg(target_os = "windows")]
use winapi;

// Oh god oh fuck I'm sorry I need to find a better way to do this
pub fn try_read_string(handle: process_memory::ProcessHandle, mut starting_offsets: Vec<usize>, buffer_size : i32) -> Result<String, std::io::Error> {
    //Read String 4 in 4 bytes
    use process_memory::*;
    use std::mem::transmute;
    let number_of_passes: f32 = buffer_size as f32 / 4.0;

    let last_offset = starting_offsets.pop().unwrap();
    let offsets: Vec<usize> = starting_offsets.clone();

    let mut string_return = String::new();
    'outer: for i in 0..(number_of_passes.ceil() as i32) {
        let player_name = DataMember::<u32>::new_offset(handle, [offsets.clone(), vec![last_offset + (4*i as usize)]].concat());
        let bytes: [u8; 4] = unsafe { transmute(player_name.read()?.to_le()) };
        for byte in bytes.iter() {
            if *byte != 0x0_u8 {
                string_return.push(*byte as char);
            } else {
                break 'outer;
            }
        }
    }

    return Ok(string_return);
}

pub fn try_read_std_string_utf8(handle: process_memory::ProcessHandle, starting_offsets: Vec<usize>) -> Result<String, std::io::Error> {
    use process_memory::*;
    let mut offset = DataMember::<u8>::new_offset(handle, starting_offsets);
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
pub fn fetch_stats(pid: process_memory::Pid) -> Result<GameData, std::io::Error> {
    use process_memory::*;
    let handle = pid.try_into_process_handle().unwrap();
    let base_address = 0x00400000;
    let game_stats_offset = 0x00500AF8;
    let game_offset = 0x00515730;

    let pb = DataMember::<f32>::new_offset(handle, vec![base_address + game_stats_offset, 0x3AC]); 
    //let timer = DataMember::<f32>::new_offset(handle, vec![base_address + game_stats_offset, 0x214]); //this is not "THE" timer, its a timer that counts actual seconds passed
    let timer = DataMember::<f32>::new_offset(handle, vec![base_address + game_stats_offset, 0x224]);
    let gems_total = DataMember::<i32>::new_offset(handle, vec![base_address + game_stats_offset, 0x244]);
    let enemies_killed = DataMember::<i32>::new_offset(handle, vec![base_address + game_stats_offset, 0x240]);
    let enemies_alive = DataMember::<i32>::new_offset(handle, vec![base_address + game_stats_offset, 0x280]);
    let daggers_fired = DataMember::<i32>::new_offset(handle, vec![base_address + game_stats_offset, 0x238]);
    let daggers_hit = DataMember::<i32>::new_offset(handle, vec![base_address + game_stats_offset, 0x23C]);
    let is_replay = DataMember::<bool>::new_offset(handle, vec![base_address + game_stats_offset, 0x41D]);
    let death_type = DataMember::<i32>::new_offset(handle, vec![base_address + game_stats_offset, 0x248]);
    let is_alive = DataMember::<bool>::new_offset(handle, vec![base_address + game_stats_offset, 0x228]);    
    let player_id = DataMember::<i32>::new_offset(handle, vec![base_address + game_stats_offset, 0xB0]);    
    let player_name = try_read_std_string_utf8(handle, vec![base_address + game_stats_offset,  0xC8]);
    let replay_player_name = try_read_std_string_utf8(handle, vec![base_address + game_stats_offset,  0x430]);

    let gems = DataMember::<i32>::new_offset(handle, vec![base_address + game_offset, 0, 0x2DC]);
    let homing = DataMember::<i32>::new_offset(handle, vec![base_address + game_offset, 0, 0x2E8]);
    let is_dead = DataMember::<i32>::new_offset(handle, vec![base_address + game_offset, 0, 0xE4]); 

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

    thread::sleep(time::Duration::from_millis(28)); // TODO - Actual timing mechanisms lol
    return Ok(data);
}

#[cfg(target_os = "windows")]
pub fn get_pid(process_name: &str) {
    
}