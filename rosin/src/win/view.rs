
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::Foundation::HWND;
use windows::core::Error;

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

impl RosinView {
    pub fn new() -> Result<RosinView, Error> {
        // I tried looking for another safe
        // or at least safer api for creating a window,
        // but for now this should hopefully do.
        let view = RosinView {
            hwnd: unsafe {
                // FIXME quick defaults, think about how to set these settings
                windows::Win32::UI::WindowsAndMessaging::CreateWindowExW(
                    windows::Win32::UI::WindowsAndMessaging::WS_EX_OVERLAPPEDWINDOW,
                    None, // window class; this is where the proc-function will be stored, so there will be a class created
                    None, // window name; FIXME should have a parameter for this
                    windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW,
                    50,   // x
                    50,   // y
                    240,  // width
                    160,  // height
                    None, // parent
                    None, // menu
                    None, // instance
                    None, // window state; this will *probably* store the final state too
                )?
            },
        };

        Ok(view)
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for RosinView {
    fn drop(&mut self) {
        
    }
}
