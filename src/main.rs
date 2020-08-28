mod mem;
mod ui;
pub mod structs;
pub mod consts;

fn cum_data(data : structs::GameData) {
    println!("\n\n\n\n\n\n\n");

    if data.is_replay {
        if data.is_dead == consts::DEATH_STATUS {
            println!("REPLAY DEAD {} ({}) - {:.4} ({})", data.replay_player_name, "lol still cant find the id", data.timer, "replay time here?");
        } else {
            println!("REPLAY {} ({}) - {:.4} ({})", data.replay_player_name, "lol still cant find the id", data.timer, "total time?");
        }
    } else {
        if data.is_alive {
            if data.timer > 0.0 { 
                println!("GAMING {} ({}) - {:.4} ({})", data.player_name, data.player_id, data.timer, data.pb);
            } else {
                if data.enemies_alive == 0 {
                    println!("LOBBY {} ({}) - {:.4} ({})", data.player_name, data.player_id, data.timer, data.pb);
                } else {
                    println!("MENU {} ({}) - {:.4} ({})", data.player_name, data.player_id, data.timer, data.pb);
                }
            }
        } 
    
        if data.is_dead == consts::DEATH_STATUS {
            println!("DEAD {} ({}) - {:.4} ({})", data.player_name, data.player_id, data.timer, data.pb);
        }
    }

    println!("GEMS:\t{}", data.gems_total);
    println!("HOMING:\t{}", data.homing);

    if data.is_dead == consts::DEATH_STATUS {
        println!("DEATH TYPE: {}", consts::DEATH_TYPES[data.death_type as usize]);
    }

}

fn main() {
    let pid = mem::get_pid("devildaggers");
    if pid == 0 {
        println!("Devil Daggers isn't running")
    } else {
        println!("{} devildaggers", pid);
        loop {
            let data = mem::fetch_stats(pid);
            match data {
                Ok(data) => cum_data(data),
                Err(e) => println!("ERROR {}", e),
            }
        }
    }
}
