pub const DEATH_STATUS : i32 = 7;

pub const V3_SURVIVAL_HASH : &str = "569fead87abf4d30fdee4231a6398051";

//Linux
pub const DD_PROCESS_LINUX : &str = "devildaggers";
pub const LINUX_BASE_ADDRESS : usize = 0x00400000;
pub const LINUX_GAME_STATS_ADDRESS : usize = LINUX_BASE_ADDRESS + 0x00500AF8;
pub const LINUX_GAME_ADDRESS : usize = LINUX_BASE_ADDRESS + 0x00515730;

pub const DEATH_TYPES : [&str; 16] = [
    "Fallen", 
    "Swarmed", 
    "Impaled", 
    "Gored", 
    "Infested", 
    "Opened", 
    "Purged",
    "Desecrated", 
    "Sacrificed", 
    "Eviscerated", 
    "Annihilated", 
    "Intoxicated",
    "Envenmonated", 
    "Incarnated", 
    "Discarnated", 
    "Barbed",
];
