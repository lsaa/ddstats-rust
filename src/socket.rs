use crate::config;
use clipboard::{ClipboardContext, ClipboardProvider};

pub fn is_offline() {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();    
    //println!("{:?}", ctx.get_contents());
    ctx.set_contents(String::from("GAMING ZONE")).unwrap();
    config::get_config().unwrap();
    println!("{:?}", ctx.get_contents().unwrap());
}