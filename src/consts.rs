//
// consts
//

pub const DEATH_STATUS: i32 = 7;

pub const V3_SURVIVAL_HASH: &str = "569fead87abf4d30fdee4231a6398051";

pub const VERSION: &str = "0.6.9";

pub const LOGO_ALT: &str = "
@@@@@@@   @@@@@@@    @@@@@@   @@@@@@@   @@@@@@   @@@@@@@   @@@@@@@
@@@@@@@@  @@@@@@@@  @@@@@@@   @@@@@@@  @@@@@@@@  @@@@@@@  @@@@@@@
@@!  @@@  @@!  @@@  !@@         @@!    @@!  @@@    @@!    !@@
!@!  @!@  !@!  @!@  !@!         !@!    !@!  @!@    !@!    !@!
@!@  !@!  @!@  !@!  !!@@!!      @!!    @!@!@!@!    @!!    !!@@!!
!@!  !!!  !@!  !!!   !!@!!!     !!!    !!!@!!!!    !!!     !!@!!!
!!:  !!!  !!:  !!!       !:!    !!:    !!:  !!!    !!:         !:!
:!:  !:!  :!:  !:!      !:!     :!:    :!:  !:!    :!:        !:!
::::: ::   :::: ::  :::: ::      ::    ::   :::     ::    :::: ::
:: :  :   :: :  :   :: : :       :      :   : :     :     :::: rust";

pub const LOGO_NEW: &str = "

████████▄  ████████▄     ▄████████     ███        ▄████████     ███        ▄████████
███   ▀███ ███   ▀███   ███    ███ ▀█████████▄   ███    ███ ▀█████████▄   ███    ███
███    ███ ███    ███   ███    █▀     ▀███▀▀██   ███    ███    ▀███▀▀██   ███    █▀
███    ███ ███    ███   ███            ███   ▀   ███    ███     ███   ▀   ███
███    ███ ███    ███ ▀███████████     ███     ▀███████████     ███     ▀███████████
███    ███ ███    ███          ███     ███       ███    ███     ███              ███
███   ▄███ ███   ▄███    ▄█    ███     ███       ███    ███     ███        ▄█    ███
████████▀  ████████▀   ▄████████▀     ▄████▀     ███    █▀     ▄████▀    ▄████████▀
v0.6.9                                                                          rust";

pub const LOGO_MINI: &str = "
____________  _____ _____ ___ _____ _____
|  _  \\  _  \\/  ___|_   _/ _ \\_   _/  ___|
| | | | | | |\\ `--.  | |/ /_\\ \\| | \\ `--.
| | | | | | | `--. \\ | ||  _  || |  `--. \\
| |/ /| |/ / /\\__/ / | || | | || | /\\__/ /
|___/ |___/  \\____/  \\_/\\_| |_/\\_/ \\____/
v0.6.9                               rust";

pub const SUBMIT_RETRY_MAX: u16 = 10;

//Linux
#[cfg(target_os = "linux")]
pub const DD_PROCESS: &str = "devildaggers";
pub const LINUX_BLOCK_START: usize = 0x0053A850;

//Windows
#[cfg(target_os = "windows")]
pub const DD_PROCESS: &str = "dd";
pub const WINDOWS_BLOCK_START: usize = 0x250DC0;

pub const DEATH_TYPES: [&str; 17] = [
    "Fallen",       // lol get rekt
    "Swarmed",      // skull 1
    "Impaled",      // get real
    "Gored",        // skull 3
    "Infested",     // spiderling
    "Opened",       // skull 4
    "Purged",       // squid 1
    "Desecrated",   // squid 2
    "Sacrificed",   // squid 3
    "Eviscerated",  // centipide
    "Annihilated",  // gigapede
    "Intoxicated",  // 3.0 ghost || spider 1
    "Envenmonated", // spider 2
    "Incarnated",   // levi
    "Discarnated",  // orb
    "Entangled",    // Replaced BARBED || thorn
    "Haunted",      // 3.1 ghost
];
