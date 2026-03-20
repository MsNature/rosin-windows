
use std::{any::Any, time::Duration};

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct2D::Common::D2D_SIZE_U;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{GetClientRect, GetDesktopWindow, GetWindowRect, GetWindowThreadProcessId, SW_NORMAL, ShowWindowAsync};
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::core::Error;

use crate::{
    kurbo::{Point, Size},
    prelude::*,
};

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
            thread_id: unsafe {
                // SAFETY: this is ran on a valid thread
                GetWindowThreadProcessId(view.hwnd(), None)
            },
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
        let current_id = unsafe {
            GetCurrentThreadId()
        };
        (self.thread_id == current_id).then(|| f(&self.view))
    }

    pub fn queue_on_thread<F>(&self, f: F)
    where
        F: FnOnce(&RosinView) + Sync + 'static
    {
        todo!()
    }

    pub fn block_on_thread<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&RosinView) -> R + Sync + 'static,
        R: Sync + 'static
    {
        todo!()
    }
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
    pub fn from_new_window<S: 'static>(desc: &WindowDesc<S>, instance: Option<HINSTANCE>, parent: Option<WindowHandle>) -> Result<RosinView, Error> {
        println!("Initializing rosin view");

        let desktop = unsafe {
            GetDesktopWindow()
        };
        
        let width = f64_to_i32(desc.size.width);
        let height = f64_to_i32(desc.size.height);

        let (x, y) = 'pos: {
            if let Some(pos) =  desc.position {
                break 'pos (f64_to_i32(pos.x), f64_to_i32(pos.y));
            }

            let desktop_size = {
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

        let view_state = Box::new(
            ViewState::new()
        );

        // I tried looking for another safe
        // or at least safer api for creating a window,
        // but for now this should hopefully do.
        let view = RosinView {
            hwnd: unsafe {
                // FIXME add safety comments
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
                    Some(parent.map(|handle| handle.0.view.view.hwnd).unwrap_or(desktop)),
                    None, // menu
                    instance,
                    Some(Box::leak(view_state) as *mut _ as *const _),
                )?
            },
        };

        unsafe {
            // SAFETY:
            //  - view.hwnd is a valid handle
            //  - SW_NORMAL is a valid show window code
            ShowWindowAsync(view.hwnd, SW_NORMAL).ok()?;
        }

        Ok(view)
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn set_input_handler(&self, _id: Option<NodeId>, _handler: Option<Box<dyn InputHandler + Send + Sync>>) {}

    pub fn get_logical_size(&self) -> Size {
        Size::ZERO
    }

    pub fn get_physical_size(&self) -> Size {
        Size::ZERO
    }

    pub fn get_position(&self) -> Point {
        Point::ZERO
    }

    pub fn get_window_state(&self) -> WindowState {
        WindowState::Normal
    }

    pub fn is_active(&self) -> bool {
        true
    }

    pub fn activate(&self) {}

    pub fn deactivate(&self) {}

    pub fn set_menu(&self, _menu: impl Into<Option<MenuDesc>>) {}

    pub fn show_context_menu(&self, _node: Option<NodeId>, _menu: MenuDesc, _pos: Point) {}

    pub fn create_window<S: Any + Sync + 'static>(&self, _desc: &WindowDesc<S>) {}

    pub fn request_close(&self) {}

    pub fn request_exit(&self) {}

    pub fn set_max_size(&self, _size: Option<impl Into<Size>>) {}

    pub fn set_min_size(&self, _size: Option<impl Into<Size>>) {}

    pub fn set_position(&self, _position: impl Into<Point>) {}

    pub fn set_resizable(&self, _resizeable: bool) {}

    pub fn set_size(&self, _size: impl Into<Size>) {}

    pub fn set_title(&self, _title: impl Into<String>) {}

    pub fn minimize(&self) {
        use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;

        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd(), SW_MINIMIZE);
        }
    }

    pub fn maximize(&self) {
        use windows::Win32::UI::WindowsAndMessaging::SW_MAXIMIZE;

        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd(), SW_MAXIMIZE);
        }
    }

    pub fn restore(&self) {
        use windows::Win32::UI::WindowsAndMessaging::SW_RESTORE;

        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(self.hwnd(), SW_RESTORE);
        }
    }

    pub fn set_cursor(&self, _cursor: CursorType) {}

    pub fn hide_cursor(&self) {}

    pub fn unhide_cursor(&self) {}

    pub fn set_clipboard_text(&self, _text: &str) {}

    pub fn get_clipboard_text(&self) -> Option<String> {
        None
    }

    pub fn open_url(&self, _url: &str) {}

    pub fn open_file_dialog(&self, _node: Option<NodeId>, _options: FileDialogOptions) {}

    pub fn save_file_dialog(&self, _node: Option<NodeId>, _options: FileDialogOptions) {}

    pub fn timer(&self, _node: Option<NodeId>, _delay: Duration) {}

    pub fn alert<C>(&self, _node: Option<NodeId>, _png_bytes: Option<&'static [u8]>, _title: &str, _details: &str, _options: &[(&'static str, C)])
    where
        C: Into<CommandId> + Copy,
    {
    }
}

impl Drop for RosinView {
    fn drop(&mut self) {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetWindowLongPtrW,
                GWLP_USERDATA,
            };

            // SAFETY: self.hwnd is a valid handle
            debug_assert!(!self.hwnd.is_invalid(), "`hwnd` at this point should be a valid window handle");
            if let Some(view_state) = std::ptr::NonNull::new(GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut ViewState) {
                // SAFETY: view_state is initialized to be valid
                std::mem::drop(Box::from_raw(view_state.as_ptr()));
            }
        }
    }
}

use windows::Win32::Graphics::Direct2D::{D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1CreateFactory, ID2D1Factory8, ID2D1HwndRenderTarget};

#[repr(C)]
pub(crate) struct ViewState {
    pub factory: ID2D1Factory8,
    pub render_target: Option<ID2D1HwndRenderTarget>
}

impl ViewState {
    fn new() -> Result<Self, Error> {
        let factory = unsafe {
            // SAFETY: all inputs are valid
            D2D1CreateFactory(D2D1_FACTORY_TYPE_MULTI_THREADED, None)?
        };

        Ok(
            ViewState {
                factory,
                render_target: None
            }
        )
    }

    /// Initalizes all the state 
    /// 
    /// SAFETY: `hwnd` must be a valid handle
    #[allow(unsafe_op_in_unsafe_fn)]
    pub unsafe fn init(&mut self, hwnd: HWND) -> Result<(), Error> {
        debug_assert!(!hwnd.is_invalid(), "`hwnd` at this point should be a valid window handle");
        self.init_graphics(hwnd)?;

        Ok(())
    }

    /// Releases all the state
    pub fn release(&mut self) {
        self.release_graphics();
    }

    /// Initializes the graphics API
    /// 
    /// SAFETY: `hwnd` must be a valid handle
    pub unsafe fn init_graphics(&mut self, hwnd: HWND) -> Result<(), Error> {
        if self.render_target.is_none() {
            debug_assert!(!hwnd.is_invalid(), "`hwnd` at this point should be a valid window handle");

            let rect = unsafe {
                let mut rect = RECT::default();
                // SAFETY:
                //  - hwnd is a valid handle
                //  - &raw mut rect is pointing to a valid memory addres
                GetClientRect(hwnd, &raw mut rect)?;
                rect
            };
            
            let render_target_properties = Default::default();
            let hwnd_render_target_properties = windows::Win32::Graphics::Direct2D::D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: size_of_rect(rect),
                presentOptions: Default::default(),
            };

            let render_target = unsafe {
                // SAFETY: All these poitners point to valid values
                self.factory.CreateHwndRenderTarget(
                    &raw const render_target_properties,
                    &raw const hwnd_render_target_properties,
                )?
            };

            let _ = self.render_target.insert(render_target);
        }

        Ok(())
    }

    /// Releases the graphics API
    pub fn release_graphics(&mut self) {
        self.render_target = None;
    }
}

impl Drop for ViewState {
    fn drop(&mut self) {
        self.release()
    }
}

fn size_of_rect(rect: RECT) -> D2D_SIZE_U {
    D2D_SIZE_U {
        width: (rect.right - rect.left) as u32,
        height: i32::abs(rect.top - rect.bottom) as u32,
    }
}
