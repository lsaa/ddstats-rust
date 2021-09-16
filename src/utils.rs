//
// cool
//

use std::cell::RefCell;
use std::time::Duration;

#[cfg(windows)]
extern crate winapi;

thread_local! {
    #[cfg(target_os = "windows")]
    static TIMER_HANDLE: RefCell<winapi::um::winnt::HANDLE> = RefCell::new(create_timer());
}

#[cfg(target_os = "windows")]
fn create_timer() -> winapi::um::winnt::HANDLE {
    use std::ptr::null_mut;
    unsafe { winapi::um::synchapi::CreateWaitableTimerW(
            null_mut(),
            true as winapi::shared::minwindef::BOOL,
            null_mut(),
        )
    }
}

#[cfg(target_os = "windows")]
pub fn sleep(d: Duration) {
    unsafe {
        use std::ptr::null_mut;
        let mut li: winapi::shared::ntdef::LARGE_INTEGER = std::mem::zeroed();
        *li.QuadPart_mut() =
            -(d.as_nanos() as i64) as winapi::ctypes::__int64 as winapi::shared::ntdef::LONGLONG;
        TIMER_HANDLE.with(|timer| {
            let mut timer = timer.borrow_mut();
            let created_timer = winapi::um::synchapi::SetWaitableTimer(
                timer.clone(),
                &li as *const _,
                0,
                None,
                null_mut(),
                false as winapi::shared::minwindef::BOOL,
            );

            winapi::um::synchapi::WaitForSingleObject(timer.clone(), winapi::um::winbase::INFINITE);
        });
    }
}

#[cfg(target_os = "linux")]
pub fn sleep(d: Duration) {
    std::thread::sleep(d);
}
