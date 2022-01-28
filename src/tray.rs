//
//  tray.rs - AAAHHAHAHAHAHAAAHAHAHAHAHHAHAHHAHA
//

use std::{time::Duration, sync::atomic::{AtomicBool, Ordering}};
use trayicon::{TrayIconBuilder, MenuBuilder, Icon};
use winreg::{RegKey, enums::HKEY_CURRENT_USER};
use crate::threads::{AAS, State, Message};
use winapi::um::{winuser::{self, SendMessageA, LoadIconA, WM_SETICON, ICON_BIG, ICON_SMALL, ICON_SMALL2, MAKEINTRESOURCEA}, libloaderapi::GetModuleHandleA};
use std::mem::MaybeUninit;
pub struct TrayIcon;

#[derive(Clone, PartialEq)]
pub enum TrayEvent {
    ShowWindow,
    HideWindow,
    Exit,
}

impl TrayIcon {
    pub async fn init(state: AAS<State>) {

        let cfg = crate::config::cfg();
        let light_theme = is_light_theme();

        if light_theme {
            unsafe { set_small_icon(3); set_big_icon(5); };
        } else {
            unsafe { set_small_icon(2); set_big_icon(4); };
        }

        if !cfg.tray_icon {
            return;
        }

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs_f32(1.));
            let mut bus_recv = state.load().msg_bus.0.subscribe();
            let bus_sndr = state.load().msg_bus.0.clone();

            let icon_dark = include_bytes!("../assets/logo16_dark.ico");
            let icon_light = include_bytes!("../assets/logo16_light.ico");
        
            let icon_dark = Icon::from_buffer(icon_dark, None, None).unwrap();
            let icon_light = Icon::from_buffer(icon_light, None, None).unwrap();
            let icon = if light_theme { icon_light } else { icon_dark };
            let (rx, tx) = std::sync::mpsc::channel();
            let rx_clone = rx.clone();
        
            let murder_tray = std::sync::Arc::new(AtomicBool::from(false));
            let murder_tray_clone = murder_tray.clone();
            std::thread::spawn(move || {
                let _tray_icon = TrayIconBuilder::new()
                    .sender(rx)
                    .icon(icon)
                    .tooltip("ddstats-rust")
                    .on_click(TrayEvent::ShowWindow)
                    .on_double_click(TrayEvent::ShowWindow)
                    .menu(
                        MenuBuilder::new()
                            .item("Hide Window", TrayEvent::HideWindow)
                            .item("Exit", TrayEvent::Exit),
                    )
                    .build()
                    .unwrap();

                loop {
                    unsafe {
                        let mut msg = MaybeUninit::uninit();
                        if murder_tray.load(Ordering::SeqCst) {
                            break;
                        }
                        let bret = winuser::GetMessageA(msg.as_mut_ptr(), 0 as _, 0, 0);
                        if bret > 0 {
                            winuser::TranslateMessage(msg.as_ptr());
                            winuser::DispatchMessageA(msg.as_ptr());
                        } else {
                            break;
                        }
                    }
                }
            });

            if cfg.hide_window_on_start {
                let _ = rx_clone.send(TrayEvent::HideWindow).unwrap();
            }

            loop {
                tokio::select! {
                    msg = bus_recv.recv() => match msg {
                        Ok(Message::ShowWindow) => { show_term(); },
                        Ok(Message::HideWindow) => { hide_term(); },
                        _ => {}
                    },
                    _elapsed = interval.tick() => {
                        match tx.try_recv() {
                            Ok(TrayEvent::Exit) => { murder_tray_clone.swap(true, Ordering::SeqCst); let _ = bus_sndr.send(Message::Exit); },
                            Ok(TrayEvent::ShowWindow) => { 
                                let _ = bus_sndr.send(Message::ShowWindow); 
                            },
                            Ok(TrayEvent::HideWindow) => { 
                                let _ = bus_sndr.send(Message::HideWindow); 
                            },
                            _ => {}
                        }
                    },
                };
            }
        });
    }
}

pub unsafe fn set_big_icon(icon_id: u16) {
    use winapi::um::wincon::GetConsoleWindow;
    let hinstance = GetModuleHandleA(std::ptr::null_mut());
    let window = GetConsoleWindow();
    let icon_res = MAKEINTRESOURCEA(icon_id);
    let icon = LoadIconA(hinstance, icon_res);
    SendMessageA(window, WM_SETICON, ICON_BIG as usize, icon as isize);
}

pub unsafe fn set_small_icon(icon_id: u16) {
    use winapi::um::wincon::GetConsoleWindow;
    let hinstance = GetModuleHandleA(std::ptr::null_mut());
    let window = GetConsoleWindow();
    let icon_res = MAKEINTRESOURCEA(icon_id);
    let icon = LoadIconA(hinstance, icon_res);
    SendMessageA(window, WM_SETICON, ICON_SMALL as usize, icon as isize);
    SendMessageA(window, WM_SETICON, ICON_SMALL2 as usize, icon as isize);
}

pub unsafe fn set_icon(icon_id: u16) {
    use winapi::um::wincon::GetConsoleWindow;
    let hinstance = GetModuleHandleA(std::ptr::null_mut());
    let window = GetConsoleWindow();
    let icon_res = MAKEINTRESOURCEA(icon_id);
    let icon = LoadIconA(hinstance, icon_res);
    SendMessageA(window, WM_SETICON, ICON_BIG as usize, icon as isize);
    SendMessageA(window, WM_SETICON, ICON_SMALL as usize, icon as isize);
    SendMessageA(window, WM_SETICON, ICON_SMALL2 as usize, icon as isize);
}

pub fn is_light_theme() -> bool {
    match is_light_theme_inner() {
        Ok(_) => true,
        Err(e) => { log::info!("{e:?}"); false },
    }
}

pub fn is_light_theme_inner() -> anyhow::Result<()> {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let cur_ver = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")?;
    let pf: u32 = cur_ver.get_value("AppsUseLightTheme")?;
    if pf == 1 {
        return Ok(())
    } else {
        anyhow::bail!("dark theme");
    };
}

pub fn hide_term() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe {GetConsoleWindow()};
    if window != ptr::null_mut() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

pub fn show_term() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::ShowWindow;

    let window = unsafe {GetConsoleWindow()};
    if window != ptr::null_mut() {
        unsafe {
            ShowWindow(window, 9);
        }
    }
}