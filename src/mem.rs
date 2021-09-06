//  lsaa | 30-08-2021
//  mem module
//  Query memory from the game process

use crate::consts::*;
use core::fmt::Write;
use process_memory::{CopyAddress, ProcessHandle};
use process_memory::{Pid, ProcessHandleExt, TryIntoProcessHandle};
use std::cell::RefCell;
use std::mem::size_of;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
#[cfg(target_os = "linux")]
use sysinfo::{ProcessExt, System, SystemExt};

pub const DATA_BLOCK_SIZE: usize = size_of::<StatsDataBlock>();
pub const STATS_FRAME_SIZE: usize = size_of::<StatsFrame>();

// based thread local so we dont have to allocate the buffer every game tick
thread_local! {
    static BLOCK_BUF: RefCell<[u8; DATA_BLOCK_SIZE]> = RefCell::new([0_u8; DATA_BLOCK_SIZE]);
    static FRAME_BUF: RefCell<[u8; STATS_FRAME_SIZE]> = RefCell::new([0_u8; STATS_FRAME_SIZE]);
}

#[rustfmt::skip] #[cfg(target_os = "linux")]
pub fn read_stats_data_block(handle: ProcessHandle, base: Option<usize>) -> Result<StatsDataBlock, std::io::Error> {
    use process_memory::*;
    let base = if base.is_none() { get_base_address(handle.0)? } else { base.unwrap() };
    let offsets = [base + LINUX_BLOCK_START, 0x0, 0x1F10];
    let pointer = handle.get_offset(&offsets)? + 0xC; // 0xC to skip the header
    BLOCK_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        handle.copy_address(pointer, buf.as_mut())?;
        let (_head, body, _tail) = unsafe { buf.as_mut().align_to::<StatsDataBlock>() };
        Ok(body[0].clone())
    })
}

#[cfg(target_os = "linux")] // CPU heaby operation, try to use it with a delay
pub fn get_proc(process_name: &str) -> Option<(String, Pid)> {
    let s = System::new_all();
    for process in s.get_process_by_name(process_name) {
        return Some((String::from(process.exe().to_str().unwrap()), process.pid()));
    }
    None
}

#[cfg(target_os = "linux")] // 1000 times better than the windows one
pub fn get_base_address(pid: Pid) -> Result<usize, std::io::Error> {
    use std::io::Read;

    let mut f = BufReader::new(File::open(format!("/proc/{}/maps", pid))?);
    let mut buf = Vec::<u8>::new();
    let mut exe = BufReader::new(File::open(format!("/proc/{}/cmdline", pid))?);
    let mut exe_buf = Vec::<u8>::new();
    exe.read_to_end(&mut exe_buf).expect("FUN");
    let exe = String::from_utf8(exe_buf).unwrap_or(String::from("HAHAHA"));
    let mut chars = exe.chars();
    chars.next_back();
    let exe = chars.as_str();
    while let Ok(_len) = f.read_until(0x0A, &mut buf) {
        let base_str = String::from_utf8(buf.clone()).expect("Couldn't decode stat");
        if base_str.contains(exe) {
            let base_str = base_str.split("-").next().unwrap();
            return Ok(usize::from_str_radix(&base_str[0..12], 16).expect("Couldn't convert base address to usize"));
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No base address"))
}

pub struct GameConnection {
    pub pid: Pid,
    pub path: String,
    pub handle: ProcessHandle,
    pub base_address: usize,
    pub last_fetch: Option<StatsBlockWithFrames>,
}

impl GameConnection {
    pub fn try_create(process_name: &str) -> Result<Self, &str> {
        let proc = get_proc(process_name);
        if proc.is_none() {
            return Err("Process not found");
        }
        let proc = proc.unwrap();
        let handle;
        let pid = proc.1;
        if pid == 0 {
            return Err("PID is 0");
        }
        handle = pid.try_into_process_handle().unwrap();
        let base_address = get_base_address(pid);
        if base_address.is_err() {
            return Err("Couldn't get base address of process");
        }
        let base_address = base_address.unwrap();
        Ok(Self {
            pid,
            handle,
            base_address,
            path: proc.0,
            last_fetch: None,
        })
    }

    pub fn dead_connection() -> Self {
        Self {
            pid: 0,
            base_address: 0,
            last_fetch: None,
            path: String::new(),
            handle: ProcessHandle::null_type(),
        }
    }

    pub fn is_alive(&self) -> bool {
        match self.handle.copy_address(self.base_address, &mut [0u8]) {
            Ok(_) => true,
            _ => false,
        }
    }

    pub fn read_stats_block(&mut self) -> Result<StatsDataBlock, std::io::Error> {
        let r = read_stats_data_block(self.handle, Some(self.base_address));
        if let Ok(data) = r {
            return Ok(data);
        }
        r
    }

    pub fn read_stats_block_with_frames(&mut self) -> Result<StatsBlockWithFrames, std::io::Error> {
        if let Ok(data) = read_stats_data_block(self.handle, Some(self.base_address)) {
            let res = StatsBlockWithFrames {
                frames: self.stat_frames_from_block(&data)?,
                block: data,
            };
            self.last_fetch = Some(res.clone());
            return Ok(res);
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No data"))
    }

    pub fn stat_frames_from_block(
        &mut self,
        block: &StatsDataBlock,
    ) -> Result<Vec<StatsFrame>, std::io::Error> {
        let (mut ptr, len) = (
            block.get_stats_pointer(),
            block.stats_frames_loaded as usize,
        );
        let mut res = Vec::with_capacity(len);
        FRAME_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            for _ in 0..len {
                self.handle.copy_address(ptr, buf.as_mut())?;
                let (_head, body, _tail) = unsafe { buf.align_to::<StatsFrame>() };
                res.push(body[0].clone());
                ptr += STATS_FRAME_SIZE;
            }
            return Ok(res);
        })
    }

    pub fn stat_frames(&self) -> Result<Vec<StatsFrame>, std::io::Error> {
        use process_memory::*;
        if let Some(last_data) = &self.last_fetch {
            let (mut ptr, len) = (
                last_data.block.get_stats_pointer(),
                last_data.block.stats_frames_loaded as usize,
            );
            let mut res = Vec::with_capacity(len);
            FRAME_BUF.with(|buf| {
                let mut buf = buf.borrow_mut();
                for _ in 0..len {
                    self.handle.copy_address(ptr, buf.as_mut())?;
                    let (_head, body, _tail) = unsafe { buf.align_to::<StatsFrame>() };
                    res.push(body[0].clone());
                    ptr += STATS_FRAME_SIZE;
                }
                return Ok(res);
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Stats not available",
            ))
        }
    }

    pub fn last_stat_frame(&self) -> Result<StatsFrame, std::io::Error> {
        use process_memory::*;
        if let Some(last_data) = &self.last_fetch {
            let (mut ptr, len) = (
                last_data.block.get_stats_pointer(),
                last_data.block.stats_frames_loaded as usize,
            );
            ptr += STATS_FRAME_SIZE * (len - 1);
            FRAME_BUF.with(|buf| {
                let mut buf = buf.borrow_mut();
                self.handle.copy_address(ptr, buf.as_mut())?;
                let (_head, body, _tail) = unsafe { buf.align_to::<StatsFrame>() };
                return Ok(body[0].clone());
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Stats not available",
            ))
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct StatsDataBlock {
    pub ddstats_version: i32,
    pub player_id: i32,
    pub username: [u8; 32],
    pub time: f32,
    pub gems_collected: i32,
    pub kills: i32,
    pub daggers_fired: i32,
    pub daggers_hit: i32,
    pub enemies_alive: i32,
    pub level_gems: i32,
    pub homing: i32,
    pub gems_despawned: i32,
    pub gems_eaten: i32,
    pub gems_total: i32,
    pub daggers_eaten: i32,
    pub per_enemy_alive_count: [i16; 17],
    pub per_enemy_kill_count: [i16; 17],
    pub is_player_alive: bool,
    pub is_replay: bool,
    pub death_type: u8,
    pub is_in_game: bool,
    pub replay_player_id: i32,
    pub replay_player_name: [u8; 32],
    pub survival_md5: [u8; 16],
    pub time_lvl2: f32,
    pub time_lvl3: f32,
    pub time_lvl4: f32,
    pub levi_down_time: f32,
    pub orb_down_time: f32,
    pub status: i32, // 0 = Intro Screen | 1 = Main Menu | 2 = InGame | 3 = DEAD
    pub max_homing: i32,
    pub time_max_homing: f32, // gets updated every gem you get even if you dont have any homing
    pub enemies_alive_max: i32, // doesn't get reset sometimes when restarting a run
    pub time_enemies_alive_max: f32,
    pub time_max: f32,       // Max time of replay / current time in-game
    padding1: [u8; 4],       // fun
    pub stats_base: [u8; 8], // Pointer to stats
    pub stats_frames_loaded: i32,
    pub stats_finished_loading: bool,
    padding2: [u8; 3],
    pub starting_hand: i32,
    pub starting_homing: i32,
    pub starting_time: f32,
    pub prohibited_mods: bool,
}

impl StatsDataBlock {
    pub fn player_username(&self) -> String {
        String::from_utf8(self.username.to_vec()).expect("Couldn't decode username string")
    }

    pub fn replay_player_username(&self) -> String {
        String::from_utf8(self.replay_player_name.to_vec())
            .expect("Couldn't decode replay player username")
    }

    pub fn level_hash(&self) -> String {
        let mut s = String::with_capacity(2 * self.survival_md5.len());
        for byte in self.survival_md5 {
            write!(s, "{:02X}", byte).expect("Couldn't decode hash byte");
        }
        return s;
    }

    pub fn get_stats_pointer(&self) -> usize {
        i64::from_le_bytes(self.stats_base) as usize
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StatsFrame {
    pub gems_collected: i32,
    pub kills: i32,
    pub daggers_fired: i32,
    pub daggers_hit: i32,
    pub enemies_alive: i32,
    pub level_gems: i32,
    pub homing: i32,
    pub gems_despawned: i32,
    pub gems_eaten: i32,
    pub gems_total: i32,
    pub daggers_eaten: i32,
    pub per_enemy_alive_count: [i16; 17],
    pub per_enemy_kill_count: [i16; 17],
}

#[derive(Debug, Default, Clone)]
pub struct StatsBlockWithFrames {
    pub block: StatsDataBlock,
    pub frames: Vec<StatsFrame>,
}

impl StatsBlockWithFrames {
    #[rustfmt::skip]
    pub fn get_frame_for_time(&self, time: f32) -> Option<&StatsFrame> {
        let real_time = time - self.block.starting_time;
        if real_time <= 0. { return None; }
        if real_time + 1. > self.frames.len() as f32 { return None; }
        return Some(&self.frames[real_time as usize]);
    }
}
