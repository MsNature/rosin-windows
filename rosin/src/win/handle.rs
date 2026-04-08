use std::{any::Any, num::NonZeroIsize, sync::Arc, time::Duration};

use raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle as RWHWindowHandle};

use crate::{
    kurbo::{Point, Size},
    platform::view::{RosinView, ThreadLockedView},
    prelude::*,
};

pub(crate) struct WindowHandle {
    /// TODO make thread safe - force to only be accesable on the main thread
    /// Most likely to some extra struct
    ///  - Do the check internally or use the same MainKey kinda initialization as the macos version?
    pub(in crate::win) view: Arc<ThreadLockedView>,
}

impl Clone for WindowHandle {
    fn clone(&self) -> Self {
        Self { view: self.view.clone() }
    }
}

impl HasWindowHandle for WindowHandle {
    fn window_handle(&self) -> Result<RWHWindowHandle<'_>, HandleError> {
        self.view
            .try_on_thread(|view| {
                let raw_ptr = view.hwnd().0;
                let handle = Win32WindowHandle::new(NonZeroIsize::new(raw_ptr as isize).ok_or(HandleError::Unavailable)?);

                unsafe { Ok(RWHWindowHandle::borrow_raw(RawWindowHandle::Win32(handle))) }
            })
            .expect("RawWindowHandle must be requested from the main thread")
    }
}

impl HasDisplayHandle for WindowHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::windows())
    }
}

impl WindowHandle {
    pub(crate) fn new(view: RosinView) -> WindowHandle {
        Self {
            view: Arc::new(ThreadLockedView::new(view)),
        }
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
        // IsWindowEnabled
        true
    }

    pub fn activate(&self) {
        // EnableWindow (set to false)
    }

    pub fn deactivate(&self) {
        // SetActiveWindow
    }

    pub fn set_menu(&self, _menu: impl Into<Option<MenuDesc>>) {
        /* call DrawMenuBar to redraw the menu bar */
    }

    pub fn show_context_menu(&self, _node: Option<NodeId>, _menu: MenuDesc, _pos: Point) {}

    pub fn create_window<S: Any + Sync + 'static>(&self, _desc: &WindowDesc<S>) {}

    pub fn request_close(&self) {}

    pub fn request_exit(&self) {}

    pub fn set_max_size(&self, _size: Option<impl Into<Size>>) {}

    pub fn set_min_size(&self, _size: Option<impl Into<Size>>) {}

    pub fn set_position(&self, position: impl Into<Point>) {
        use crate::platform::view::f64_to_i32;
        use windows::Win32::UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos};

        let position = position.into();

        unsafe {
            // TEMP SAFETY: read the **WIN32 THREAD SAFETY** comment from `.../rosin/src/win/mod.rs`
            // SAFETY: SWP_ASYNCWINDOWPOS guarantees thread safety
            self.view.try_on_trust(move |view| {
                // SAFETY:
                //  - view.hwnd() is a valid handle
                //  - None is a valid optional handle
                //  - SWP_ASYNCWINDOWPOS, SWP_NOSIZE and SWP_NOZORDER are a valid value when xor'ed
                let _res =
                    SetWindowPos(view.hwnd(), None, f64_to_i32(position.x), f64_to_i32(position.y), 0, 0, SWP_ASYNCWINDOWPOS | SWP_NOSIZE | SWP_NOZORDER);

                #[cfg(debug_assertions)]
                if let Err(err) = _res {
                    eprintln!("`set_size` failed to change the size for `{hwnd:?}` \"{err}\": {err:#?}", hwnd = view.hwnd())
                }
            })
        }
    }

    pub fn set_resizable(&self, resizeable: bool) {
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowLongPtrW,
            GetWindowLongPtrW,
            GWL_STYLE,
            WS_SIZEBOX,
        };

        unsafe {
            // TEMP SAFETY: read the **WIN32 THREAD SAFETY** comment from `.../rosin/src/win/mod.rs`
            self.view.try_on_trust(
                move |view| {
                    let style = GetWindowLongPtrW(view.hwnd(), GWL_STYLE);
                    
                    match resizeable {
                        true => SetWindowLongPtrW(view.hwnd(), GWL_STYLE, style | WS_SIZEBOX.0 as isize),
                        false => SetWindowLongPtrW(view.hwnd(), GWL_STYLE, style & !(WS_SIZEBOX.0 as isize)),
                    };
                }
            )
        }
    }

    pub fn set_size(&self, size: impl Into<Size>) {
        use crate::platform::view::f64_to_i32;
        use windows::Win32::UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SWP_NOMOVE, SWP_NOZORDER, SetWindowPos};

        let size = size.into();

        unsafe {
            // TEMP SAFETY: read the **WIN32 THREAD SAFETY** comment from `.../rosin/src/win/mod.rs`
            // SAFETY: SWP_ASYNCWINDOWPOS guarantees thread safety
            self.view.try_on_trust(move |view| {
                // SAFETY:
                //  - view.hwnd() is a valid handle
                //  - None is a valid optional handle
                //  - SWP_ASYNCWINDOWPOS, SWP_NOMOVE and SWP_NOZORDER are a valid value when xor'ed
                let _res =
                    SetWindowPos(view.hwnd(), None, 0, 0, f64_to_i32(size.width), f64_to_i32(size.height), SWP_ASYNCWINDOWPOS | SWP_NOMOVE | SWP_NOZORDER);

                #[cfg(debug_assertions)]
                if let Err(err) = _res {
                    eprintln!("`set_size` failed to change the size for `{hwnd:?}` \"{err}\": {err:#?}", hwnd = view.hwnd())
                }
            })
        }
    }

    // NOTE: maybe add a `set_position_and_size` method for microptimizations?

    pub fn set_title(&self, _title: impl Into<String>) {}

    pub fn minimize(&self) {
        self.view
            .try_on_thread(RosinView::minimize)
            .expect("Temporary crash fail when `minimize` is ran");
    }

    pub fn maximize(&self) {
        self.view
            .try_on_thread(RosinView::maximize)
            .expect("Temporary crash fail when `minimize` is ran");
    }

    pub fn restore(&self) {
        self.view
            .try_on_thread(RosinView::restore)
            .expect("Temporary crash fail when `minimize` is ran");
    }

    pub fn set_cursor(&self, _cursor: CursorType) {
        unsafe {
            // TEMP SAFETY: read the **WIN32 THREAD SAFETY** comment from `.../rosin/src/win/mod.rs`
            // FUTURE SAFETY: Gonna use async functions for queuing setting the cursor within the messege queue
            // CURRENT SAFETY: nothing is done with `hwnd`
            self.view.try_on_trust(|view| {
                let hwnd = view.hwnd();
                todo!("yet")
            })
        }
    }

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
