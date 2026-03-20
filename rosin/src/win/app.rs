use std::sync::OnceLock;
use std::{cell::RefCell, rc::Rc};

use crate::{
    prelude::*,
    platform::view::RosinView,
};

static APP_STARTED: OnceLock<()> = OnceLock::new();

pub(crate) struct AppLauncher<S: Sync + 'static> {
    windows: Vec<WindowDesc<S>>,
    translation_map: Option<TranslationMap>,
    wgpu_config: WgpuConfig,
    state: Option<Rc<RefCell<S>>>,

    #[cfg(all(feature = "hot-reload", debug_assertions))]
    hot_reloader: RefCell<Option<crate::mac::hot::HotReloader>>,
}

impl<S: Sync + 'static> AppLauncher<S> {
    pub fn new(window: WindowDesc<S>) -> Self {
        Self {
            windows: vec![window],
            translation_map: None,
            wgpu_config: WgpuConfig::default(),
            state: None,

            #[cfg(all(feature = "hot-reload", debug_assertions))]
            hot_reloader: RefCell::new(None),
        }
    }

    pub fn with_wgpu_config(mut self, config: WgpuConfig) -> Self {
        self.wgpu_config = config;
        self
    }

    pub fn add_window(mut self, window: WindowDesc<S>) -> Self {
        self.windows.push(window);
        self
    }

    // No hot-reload, no serde requirement
    #[cfg(not(all(feature = "hot-reload", debug_assertions)))]
    pub fn run(self, state: S, translation_map: TranslationMap) -> Result<(), LaunchError> {
        self.run_impl(state, translation_map)
    }

    // Yes hot-reload, yes serde requirement
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    pub fn run(mut self, mut _state: S, _translation_map: TranslationMap) -> Result<(), LaunchError>
    where
        S: serde::Serialize + serde::de::DeserializeOwned + crate::typehash::TypeHash + 'static,
    {
        todo!("Running debug mode with the `hot-reload` feature enabled.")
    }

    fn run_impl(mut self, state: S, translation_map: TranslationMap) -> Result<(), LaunchError> {
        if APP_STARTED.set(()).is_err() {
            return Err(LaunchError::AlreadyStarted);
        }
        
        use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, TranslateMessage, DispatchMessageW, MSG};

        println!("Running {}", self.windows.first().unwrap().title.as_deref().unwrap_or("rosin app")); // TODO remove

        let instance = unsafe {
            // TODO: failiure to create a window should **not** cause a crash
            // doing it this way so it's not ignored
            windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap_or_else(
                |err| panic!("Failed to get windows instance: \"{err}\"")
            ).into()
        };

        let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSW {
            lpszClassName: crate::platform::proc_fn::ROSIN_CLASS,
            lpfnWndProc: Some(crate::platform::proc_fn::proc),
            hInstance: instance,
            ..Default::default()
        };

        let _class_atom = unsafe {
            windows::Win32::UI::WindowsAndMessaging::RegisterClassW( &raw const wc )
        };

        self.state = Some(Rc::new(RefCell::new(state)));
        self.translation_map = Some(translation_map);

        for window_desc in self.windows.iter() {
            // TODO: failiure to create a window should **not** cause a crash
            //       doing it this way so it's not ignored
            // NOTE: `None` here means a `GetDesktopHwnd()` is needlessly called for each window
            //       might cause (in)significant lag
            RosinView::from_new_window(window_desc, Some(instance), None).unwrap_or_else(
                |err| panic!("Failed to create window: \"{err}\"")
            );
        }

        let mut message = MSG::default();

        println!("Starting Rosin Loop");

        // TODO decide: how threading works here (since windows *can* come from diferent threads)
        loop {
            let result = unsafe {
                GetMessageW(&raw mut message, None, 0, 0)
            };

            println!("{result:?}; {message:?}");

            if result.0 <= 0 {
                // if let Err(_err) = result.ok() {
                //     // maybe handle err
                // }
                break;
            }

            unsafe {
                // TODO decide: ignore or `continue`?
                #[expect(unused_must_use)]
                TranslateMessage(&raw mut message);
                let _ = DispatchMessageW(&raw mut message);
            }
        };

        Ok(())
    }
}
