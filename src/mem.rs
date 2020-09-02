use crate::structs::{GameDataMembers, GameDataMembersRetrieval};
use std::mem::transmute;
use crate::app;
use std::sync::atomic::{AtomicBool, AtomicI32};

#[cfg(target_os = "linux")]
use sysinfo::{ProcessExt, System, SystemExt};
use crate::consts::{LINUX_GAME_ADDRESS, LINUX_GAME_STATS_ADDRESS};

#[cfg(target_os = "windows")]
use winapi;

pub fn try_read_std_string_utf8(handle: process_memory::ProcessHandle, starting_offsets: Vec<usize>) -> Result<String, std::io::Error> {
    use process_memory::*;
    let offset = handle.get_offset(&starting_offsets);
    let offset_len = handle.get_offset(&starting_offsets)? - 8;
    let mut len = [0_u8; 4];
    handle.copy_address(offset_len, &mut len)?;
    let mut bytes = vec![0_u8; unsafe {transmute::<[u8; 4], i32>(len)} as usize];
    handle.copy_address(offset?, &mut bytes)?;
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
pub fn fetch_stats(app: &mut app::App) -> Result<GameDataMembersRetrieval, std::io::Error> {
    use process_memory::*;
    let handle = app.process_handle.unwrap();
    let dm = GameDataMembers {
        pb: DataMember::<f32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x3AC]),
        timer: DataMember::<f32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x224]),
        gems_total: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x244]),
        enemies_killed: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x240]),
        enemies_alive: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x280]),
        daggers_fired: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x238]),
        daggers_hit: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x23C]),
        is_replay: DataMember::<bool>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x41D]),
        death_type: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x248]),
        is_alive: DataMember::<bool>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0x228]),
        player_id: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0xB0]),
        replay_player_id: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_STATS_ADDRESS, 0xB0]),
        gems_upgrade: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_ADDRESS, 0, 0x2DC]),
        homing: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_ADDRESS, 0, 0x2E8]),
        is_dead: DataMember::<i32>::new_offset(handle, vec![LINUX_GAME_ADDRESS, 0, 0xE4]),
    };
    //let levi_dead = DataMember::<i32>::new_offset(handle, vec![0x915a58]);

    let player_name = try_read_std_string_utf8(handle, vec![LINUX_GAME_STATS_ADDRESS,  0xC8]);
    let replay_player_name = try_read_std_string_utf8(handle, vec![LINUX_GAME_STATS_ADDRESS,  0x430]);

    let data = GameDataMembersRetrieval {
        timer: dm.timer.read()?,
        pb: dm.pb.read()?,
        gems_total: AtomicI32::new(dm.gems_total.read()?),
        gems_upgrade: AtomicI32::new(dm.gems_upgrade.read()?),
        is_dead: AtomicI32::new(dm.is_dead.read()?),
        daggers_fired: AtomicI32::new(dm.daggers_fired.read()?),
        daggers_hit: AtomicI32::new(dm.daggers_hit.read()?),
        player_id: AtomicI32::new(dm.player_id.read()?),
        player_name: player_name?,
        replay_player_id: AtomicI32::new(1),
        replay_player_name: replay_player_name?,
        is_alive: AtomicBool::new(dm.is_alive.read()?),
        is_replay: AtomicBool::new(dm.is_replay.read()?),
        homing: AtomicI32::new(dm.homing.read()?),
        death_type: AtomicI32::new(dm.death_type.read()?),
        enemies_alive: AtomicI32::new(dm.enemies_alive.read()?),
        enemies_killed: AtomicI32::new(dm.enemies_killed.read()?),
    };

    return Ok(data);
}

#[cfg(target_os = "windows")]
pub fn get_pid(process_name: &str) {
    
}