use crate::config;
use crate::consts;
//use serde_json::json;
use isahc::prelude::*;
use crate::structs::{MotdRespose, MotdRequest, GameRecording, SubmitRunResponse};

pub fn get_motd() {
    let cfg = config::get_config().unwrap();
    let url = cfg.host + "/api/v2/client_connect";

    let request_body = serde_json::to_vec(&MotdRequest {
        version: consts::VERSION.to_string()
    }).unwrap();

    let res = isahc::prelude::Request::post(url)
        .header("content-type", "application/json")
        .body(request_body).unwrap().send();
    
    if res.is_ok() {
        let res = res.unwrap().json::<MotdRespose>();
        let res = res.unwrap();
        log::info!("MOTD: {:?}", res.motd);
    }
}

pub fn submit_run(recording: GameRecording) {
    let cfg = config::get_config().unwrap();
    let url = cfg.host + "/api/submit_game";
    let request_body = serde_json::to_vec(&recording).unwrap();
    
    if !cfg.offline && recording.replay_player_id == 0 {
        let mut res = isahc::prelude::Request::post(url)
                .header("content-type", "application/json")
                .body(request_body).unwrap().send().unwrap();
    
        if res.status() == 200 {
            let res = res.json::<SubmitRunResponse>();
            let res = res.unwrap();
            log::info!("Submit Test: {:?}", res.message);
        } else {
            log::error!("ERROR ON SUBMIT RUN {:#?}", res);
        }
    } else {
        log::info!("\n\n\n NET SUBMISSION: \n {:#?}", recording);
    }
}