
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, GetWindowRect, GetWindowThreadProcessId, ShowWindowAsync, SW_NORMAL};
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::core::Error;

use crate::desc::WindowDesc;

/// A struct for safely locking the use of a View on a single thread
pub(crate) struct ThreadLockedView {
    view: RosinView,
    thread_id: u32,
}

impl ThreadLockedView {
    // this is the best that can be done without knowing the "main" thread from what I've reaserched
    // It's a vary leaky and not-good implementation but windows does not provide the api to know
    // which thread is the "main" thread without acces to the "main" thread.
    //
    // If there is a way to detect the "main" thread then it should be 110% added
    pub fn new(view: RosinView) -> ThreadLockedView {
        Self {
            thread_id: unsafe { GetWindowThreadProcessId(view.hwnd(), None) },
            view
        }
    }

    /// Tries to execute immidietly
    /// 
    /// Returns [`None`] if it's not found on the original thread
    pub fn try_on_thread<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&RosinView) -> R
    {
        let current_id = unsafe { GetCurrentThreadId() };
        (self.thread_id == current_id).then(|| f(&self.view))
    }

    // pub fn queue_on_thread
    // pub fn block_on_thread
}

// SAFETY: You can only acces the !Send View if it's on the original thread
unsafe impl Send for ThreadLockedView {}

// SAFETY: You can only acces the !Sync View if it's on the original thread
unsafe impl Sync for ThreadLockedView {}

pub(crate) struct RosinView {
    hwnd: HWND,
}

// here to easely change the impl later
fn f64_to_i32(f: f64) -> i32 {
    f as i32
}

// TODO: implement all the unused (menu) stuff
impl RosinView {
    pub fn create_window<S: 'static>(desc: &WindowDesc<S>, instance: Option<HINSTANCE>) -> Result<RosinView, Error> {
        println!("Initializing rosin view");

        let width = f64_to_i32(desc.size.width);
        let height = f64_to_i32(desc.size.height);

        let (x, y) = 'pos: {
            if let Some(pos) =  desc.position {
                break 'pos (f64_to_i32(pos.x), f64_to_i32(pos.y));
            }

            let desktop_size = {
                let desktop = unsafe { GetDesktopWindow() };
                let mut rect = Default::default();
                unsafe {
                    GetWindowRect(desktop, &raw mut rect)?;
                }
                rect
            };
    
            let desktop_width = desktop_size.right - desktop_size.left;
            let desktop_height = desktop_size.bottom - desktop_size.top;

            let x = (desktop_width - width) / 2;
            let y = (desktop_height - height) / 2;

            (x, y)
        };

        // I tried looking for another safe
        // or at least safer api for creating a window,
        // but for now this should hopefully do.
        let view = RosinView {
            hwnd: unsafe {
                // FIXME quick defaults, think about how to set these settings
                windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
                    windows::Win32::UI::WindowsAndMessaging::WS_EX_OVERLAPPEDWINDOW,
                    crate::platform::proc_fn::ROSIN_CLASS,
                    // TODO check: From my testing this "clones" the string; Not 100% sure if it's sound or UB though
                    desc.title
                        .as_deref()
                        .map(AsRef::<std::ffi::OsStr>::as_ref)
                        .map(std::os::windows::ffi::OsStrExt::encode_wide)
                        .map(Iterator::collect::<Vec<u16>>)
                        .as_ref()
                        .map(Vec::as_ptr)
                        .map(windows::core::PCWSTR::from_raw)
                        .as_ref(),
                    windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW,
                    x,
                    y,
                    width,
                    height,
                    None, // parent
                    None, // menu
                    instance,
                    None, // window state; this will *probably* store the final state too
                )?
            },
        };

        unsafe {
            ShowWindowAsync(view.hwnd, SW_NORMAL);
        }

        Ok(view)
    }

    pub fn from_hwnd(hwnd: HWND) -> RosinView {
        RosinView { hwnd }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

// impl Drop for RosinView {
//     fn drop(&mut self) {
        
//     }
// }
