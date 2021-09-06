//
// consts
//

pub const DEATH_STATUS: i32 = 7;

pub const V3_SURVIVAL_HASH: &str = "569fead87abf4d30fdee4231a6398051";

pub const VERSION: &str = "0.6.8";

pub const LOGO: &str = "███████████████████████████████████████████████████████████████████████
█ @@@@@@@   @@@@@@@    @@@@@@   @@@@@@@   @@@@@@   @@@@@@@   @@@@@@@  █
█ @@@@@@@@  @@@@@@@@  @@@@@@@   @@@@@@@  @@@@@@@@  @@@@@@@  @@@@@@@   █
█ @@!  @@@  @@!  @@@  !@@         @@!    @@!  @@@    @@!    !@@       █
█ !@!  @!@  !@!  @!@  !@!         !@!    !@!  @!@    !@!    !@!       █
█ @!@  !@!  @!@  !@!  !!@@!!      @!!    @!@!@!@!    @!!    !!@@!!    █
█ !@!  !!!  !@!  !!!   !!@!!!     !!!    !!!@!!!!    !!!     !!@!!!   █
█ !!:  !!!  !!:  !!!       !:!    !!:    !!:  !!!    !!:         !:!  █
█ :!:  !:!  :!:  !:!      !:!     :!:    :!:  !:!    :!:        !:!   █
█ ::::: ::   :::: ::  :::: ::      ::    ::   :::     ::    :::: ::   █
█ :: :  :   :: :  :   :: : :       :      :   : :     :     :::: rust █
███████████████████████████████████████████████████████████████████████";

pub const LOGO_NEW: &str = "
████████▄  ████████▄     ▄████████     ███        ▄████████     ███        ▄████████
███   ▀███ ███   ▀███   ███    ███ ▀█████████▄   ███    ███ ▀█████████▄   ███    ███
███    ███ ███    ███   ███    █▀     ▀███▀▀██   ███    ███    ▀███▀▀██   ███    █▀ㅤ
███    ███ ███    ███   ███            ███   ▀   ███    ███     ███   ▀   ███ㅤㅤㅤㅤ
███    ███ ███    ███ ▀███████████     ███     ▀███████████     ███     ▀███████████
███    ███ ███    ███          ███     ███       ███    ███     ███              ███
███   ▄███ ███   ▄███    ▄█    ███     ███       ███    ███     ███        ▄█    ███
████████▀  ████████▀   ▄████████▀     ▄████▀     ███    █▀     ▄████▀    ▄████████▀ㅤ
v0.6.8                                                                          rust";

//Linux
#[cfg(target_os = "linux")]
pub const DD_PROCESS: &str = "devildaggers";
pub const LINUX_BLOCK_START: usize = 0x0052BF90;

//Windows
#[cfg(target_os = "windows")]
pub const DD_PROCESS: &str = "dd";
pub const WINDOWS_BLOCK_START: usize = 0x0052BF90;

pub const DEATH_TYPES: [&str; 18] = [
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
    "Entangled",
    "Haunted",
];
