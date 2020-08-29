use crate::structs::GameData;
use process_memory::{ProcessHandle, Pid, TryIntoProcessHandle, CopyAddress};
use clokwerk::Scheduler;
use crate::mem;
use crate::consts;
use crate::utils;
use crate::structs::State;

pub struct App {
    pub state: State,
    pub previous_data: Option<GameData>,
    pub game_pid: Option<Pid>,
    pub process_handle: Option<ProcessHandle>,
    pub scheduler: Scheduler,
}

impl App {
    pub fn tick(&mut self) {
        self.scheduler.run_pending();
        if self.process_handle.is_some() && self.is_handle_valid() {
            if !self.handle_connect_test() {return;}
            let data = mem::fetch_stats(self.process_handle.unwrap());
            match data {
                Ok(data) => self.process_data(data),
                Err(e) => println!("ERROR {}", e),
            }
        } else {
            self.connect_to_game();
        }
    }

    fn handle_connect_test(&mut self) -> bool{
        if self.is_game_ready() && self.state == State::Connecting {
            self.state = State::Menu;
            println!("Connected");
        } else {
            if self.state == State::NotConnected {
                self.state = State::Connecting;
                println!("Game found, connecting...");
            }
            if self.state == State::Connecting {
                return false;
            }
        }
        return true;
    }

    fn process_data(&mut self, data: GameData) {
        self.previous_data = Some(data.clone());
        self.resolve_status();
        utils::cum_data(data);
    }

    fn resolve_status(&mut self) {
        let data = self.previous_data.as_ref().unwrap();
        if data.is_replay {
            self.state = State::Replay;
            return;
        } else {
            if data.is_alive {
                if data.timer > 0.0 { 
                    self.state = State::Playing;
                    return;
                } else {
                    if data.enemies_alive == 0 {
                        self.state = State::Lobby;
                        return;
                    } else {
                        self.state = State::Menu;
                        return;
                    }
                }
            } 
        
            if data.is_dead == consts::DEATH_STATUS {
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
        self.state = State::NotConnected;
        self.game_pid = None;
        self.process_handle = None;
        return false;
    }

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
        }
    }
}