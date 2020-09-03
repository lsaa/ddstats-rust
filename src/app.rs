use crate::structs::{GameData, GameDataMembersRetrieval, DataSlices};
use process_memory::{ProcessHandle, Pid, TryIntoProcessHandle, CopyAddress};
use crate::mem;
use crate::consts;
use crate::utils;
use std::sync::atomic::{AtomicI32, Ordering};
use crate::structs::State;
use crate::structs::GameDataMembers;

#[derive(Debug)]
pub struct App {
    pub state: State,
    pub data: Option<GameData>,
    pub game_pid: Option<Pid>,
    pub process_handle: Option<ProcessHandle>,
    pub data_members: Option<GameDataMembers>
}

unsafe impl Send for App {}

impl App {
    pub fn tick(&mut self) {
        if self.process_handle.is_some() && self.is_handle_valid() {
            if !self.handle_connect_test() {return;}
            let data = mem::fetch_stats(self);
            match data {
                Ok(data) => self.preprocess_data(data),
                Err(e) => log::error!("{}", e),
            }
        } else {
            self.connect_to_game();
        }
    }

    fn handle_connect_test(&mut self) -> bool{
        if self.is_game_ready() && self.state == State::Connecting {
            self.state = State::Menu;
            log::info!("Connected to the game")
        } else {
            if self.state == State::NotConnected {
                self.state = State::Connecting;
                log::info!("Game found, connecting...");
            }
            if self.state == State::Connecting {
                return false;
            }
        }
        return true;
    }

    fn preprocess_data(&mut self, data: GameDataMembersRetrieval) {
        if self.data.as_ref().is_none() {
            log::info!("Created GameData");
            self.data = Some(GameData {
                last_fetch_data: Some(data),
                data_slices: DataSlices::new(),
                homing_max: AtomicI32::new(0),
                homing_max_time: 0.0,
                accuracy: 0.0,
                level_2_time: 0.0,
                level_3_time: 0.0,
                level_4_time: 0.0,
                last_recording: -1.0,
                enemies_alive_max_per_second: AtomicI32::new(0),
                homing_max_per_second: AtomicI32::new(0),
            });
        } else {
            self.process_data(data);
        }
    }

    fn process_data(&mut self, data: GameDataMembersRetrieval) {
        self.resolve_status();

        let mut current_data = self.data.as_mut().unwrap();
        if current_data.last_fetch_data.as_ref().is_none() {println!("{:?}", self.game_pid) ;return;}
        let last_data = current_data.last_fetch_data.as_ref().unwrap();
        //let data = current_data.last_fetch_data.as_ref().unwrap();

        if data.timer < last_data.timer {
            if current_data.data_slices.timer.len() != 0 {
                log::info!("Submitting Run...");
                current_data.log_run();
            }
            self.data = None;
            return;
        }


        if  data.daggers_fired.load(Ordering::SeqCst) > 0 {
            current_data.accuracy = (data.daggers_hit.load(Ordering::SeqCst)as f32 / data.daggers_fired.load(Ordering::SeqCst) as f32) * 100.0;
        } else {
            current_data.accuracy = 0.0;
        }
        
        if data.homing.load(Ordering::SeqCst) > current_data.homing_max.load(Ordering::SeqCst) {
            current_data.homing_max.store(data.homing.load(Ordering::SeqCst), Ordering::SeqCst);
            current_data.homing_max_time = data.timer;
        }

        if data.gems_upgrade.load(Ordering::SeqCst) >= 10 && current_data.level_2_time == 0.0 {
            current_data.level_2_time = data.timer;
        }

        if data.gems_upgrade.load(Ordering::SeqCst) == 70 && current_data.level_3_time == 0.0 {
            current_data.level_3_time = data.timer;
        }
        
        if data.gems_upgrade.load(Ordering::SeqCst) == 71 && current_data.level_4_time == 0.0 {
            current_data.level_4_time = data.timer;
        }

        if self.state == State::Playing && (data.timer - current_data.last_recording).floor() >= 1.0 { 
            let homing = if *current_data.homing_max_per_second.get_mut() > data.homing.load(Ordering::SeqCst) 
                { *current_data.homing_max_per_second.get_mut() } else { data.homing.load(Ordering::SeqCst) };
            let enemies_alive = if *current_data.enemies_alive_max_per_second.get_mut() > data.enemies_alive.load(Ordering::SeqCst)
                { *current_data.enemies_alive_max_per_second.get_mut() } else { data.enemies_alive.load(Ordering::SeqCst) };
            let enemies_alive = if data.timer < 1.0 { 0 } else { enemies_alive };

            current_data.data_slices.total_gems.push(AtomicI32::new(data.gems_total.load(Ordering::SeqCst)));
            current_data.data_slices.timer.push(data.timer);
            current_data.data_slices.daggers_fired.push(AtomicI32::new(data.daggers_fired.load(Ordering::SeqCst)));
            current_data.data_slices.daggers_hit.push(AtomicI32::new(data.daggers_hit.load(Ordering::SeqCst)));
            current_data.data_slices.homing.push(AtomicI32::new(homing));
            current_data.data_slices.enemies_killed.push(AtomicI32::new(data.enemies_killed.load(Ordering::SeqCst)));
            current_data.data_slices.enemies_alive.push(AtomicI32::new(enemies_alive));

            current_data.last_recording = data.timer;
        }

        current_data.last_fetch_data = Some(data);
        utils::cum_data(current_data);
    }

    fn resolve_status(&mut self) {
        let data_opt = self.data.as_ref();
        if data_opt.is_none() {return;}
        let fetch_data = &data_opt.unwrap().last_fetch_data;
        if fetch_data.is_none() {return;}
        let data = fetch_data.as_ref().unwrap();
        if data.is_replay.load(Ordering::SeqCst) {
            self.state = State::Replay;
            return;
        } else {
            if data.is_alive.load(Ordering::SeqCst) {
                if data.timer > 0.0 { 
                    self.state = State::Playing;
                    return;
                } else {
                    if data.enemies_alive.load(Ordering::SeqCst) == 0 {
                        self.state = State::Lobby;
                        return;
                    } else {
                        self.state = State::Menu;
                        return;
                    }
                }
            } 
        
            if data.is_dead.load(Ordering::SeqCst) == consts::DEATH_STATUS {
                self.state = State::Dead;
                return;
            }
        }

    }

    fn is_handle_valid(&mut self) -> bool {
        if self.process_handle.is_some() {
            let mut byte = [0u8];
            let status = self.process_handle.unwrap().copy_address(0x00400000, &mut byte);
            match status {
                Ok(_) => return true,
                Err(e) => return self.game_disconnected(e),
            }
        }
        return false;
    }

    fn game_disconnected(&mut self, _: std::io::Error) -> bool {
        println!("Game Disconnected");
        log::info!("Game Disconnected");
        self.state = State::NotConnected;
        self.game_pid = None;
        self.process_handle = None;
        return false;
    }

    #[cfg(target_os = "linux")]
    fn is_game_ready(&mut self) -> bool {
        if self.state == State::Connecting && self.process_handle.is_some() {
            return self.process_handle.unwrap().get_offset(&[consts::LINUX_GAME_ADDRESS, 0, 0x2DC]).is_ok();
        }
        return false;
    }

    #[cfg(target_os = "linux")]
    pub fn connect_to_game(&mut self) {
        let pid = mem::get_pid(consts::DD_PROCESS_LINUX);
        if pid != 0 {
            self.game_pid = Some(pid);
            self.process_handle = Some(pid.try_into_process_handle().unwrap());
            mem::setup_data_members(self);
        }
    }

    #[cfg(target_os = "windows")]
    fn is_game_ready(&mut self) -> bool {

    }

    #[cfg(target_os = "windows")]
    pub fn connect_to_game(&mut self) {

    }
}