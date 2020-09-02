use crate::structs::GameData;
use crate::consts;
use std::sync::atomic::{Ordering};

pub fn cum_data(d : &mut GameData) {
    println!("\n\n\n\n\n\n\n");

    if d.last_fetch_data.as_mut().is_none() {return;}
    let data = d.last_fetch_data.as_ref().unwrap();
    if data.is_replay.load(Ordering::SeqCst) {
        if data.is_dead.load(Ordering::SeqCst) == consts::DEATH_STATUS {
            println!("REPLAY DEAD {} ({}) - {:.4} ({})", data.replay_player_name, "lol still cant find the id", data.timer, "replay time here?");
        } else {
            println!("REPLAY {} ({}) - {:.4} ({})", data.replay_player_name, "lol still cant find the id", data.timer, "total time?");
        }
    } else {
        if data.is_alive.load(Ordering::SeqCst) {
            if data.timer > 0.0 { 
                println!("GAMING {} ({}) - {:.4} ({})", data.player_name, data.player_id.load(Ordering::SeqCst), data.timer, data.pb);
            } else {
                if data.enemies_alive.load(Ordering::SeqCst) == 0 {
                    println!("LOBBY {} ({}) - {:.4} ({})", data.player_name, data.player_id.load(Ordering::SeqCst), data.timer, data.pb);
                } else {
                    println!("MENU {} ({}) - {:.4} ({})", data.player_name, data.player_id.load(Ordering::SeqCst), data.timer, data.pb);
                }
            }
        } 
    
        if data.is_dead.load(Ordering::SeqCst) == consts::DEATH_STATUS {
            println!("DEAD {} ({}) - {:.4} ({})", data.player_name, data.player_id.load(Ordering::SeqCst), data.timer, data.pb);
        }
    }

    println!("ACCURACY:\t{:.4}", d.accuracy);
    //println!("SLICES SIZE {}", d.data_slices.timer.len());
    println!("GEMS:\t\t{}", data.gems_total.load(Ordering::SeqCst));
    println!("HOMING:\t\t{} [MAX: {} AT {:.4}]", data.homing.load(Ordering::SeqCst), d.homing_max.load(Ordering::SeqCst), d.homing_max_time);
    println!("L2: {:.4} L3: {:.4} L4: {:.4}", d.level_2_time, d.level_3_time, d.level_4_time);

    if data.is_dead.load(Ordering::SeqCst) == consts::DEATH_STATUS {
        println!("DEATH TYPE: {}", consts::DEATH_TYPES[data.death_type.load(Ordering::SeqCst) as usize]);
    }

}