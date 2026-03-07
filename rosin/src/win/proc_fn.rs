
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
use windows::core::{PCWSTR, w};

use crate::{
    platform::{view::RosinView, handle::WindowHandle},
};

pub(crate) const ROSIN_CLASS: PCWSTR = w!("RosinGUI Windows Class");

pub(crate) unsafe extern "system" fn proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let view = RosinView::from_hwnd(hwnd);
    let _handle = WindowHandle::new(view);

    #[allow(unsafe_op_in_unsafe_fn)]
    DefWindowProcW(hwnd, msg, w_param, l_param)
}
