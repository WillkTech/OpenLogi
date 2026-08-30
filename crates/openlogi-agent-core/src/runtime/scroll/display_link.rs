//! Display vsync synchronization for smooth scroll decay on macOS (`CVDisplayLink`).

#[cfg(all(target_os = "macos", not(test)))]
#[expect(unsafe_code, reason = "CoreVideo CVDisplayLink C APIs require raw FFI")]
mod imp {
    use std::ffi::c_void;
    use std::sync::Arc;
    use std::thread::Thread;

    type CVDisplayLinkRef = *mut c_void;
    type CVReturn = i32;
    type CVOptionFlags = u64;

    #[repr(C)]
    struct CVTimeStamp {
        version: u32,
        video_time_scale: i32,
        video_time: i64,
        host_time: u64,
        rate_scalar: f64,
        video_refresh_period: i64,
        smpte_time: [u8; 16],
        flags: u64,
        reserved: u64,
    }

    type CVDisplayLinkOutputCallback = unsafe extern "C" fn(
        display_link: CVDisplayLinkRef,
        in_now: *const CVTimeStamp,
        in_output_time: *const CVTimeStamp,
        flags_in: CVOptionFlags,
        flags_out: *mut CVOptionFlags,
        display_link_context: *mut c_void,
    ) -> CVReturn;

    #[link(name = "CoreVideo", kind = "framework")]
    unsafe extern "C" {
        fn CVDisplayLinkCreateWithActiveCGDisplays(
            display_link_out: *mut CVDisplayLinkRef,
        ) -> CVReturn;
        fn CVDisplayLinkSetOutputCallback(
            display_link: CVDisplayLinkRef,
            callback: CVDisplayLinkOutputCallback,
            user_info: *mut c_void,
        ) -> CVReturn;
        fn CVDisplayLinkStart(display_link: CVDisplayLinkRef) -> CVReturn;
        fn CVDisplayLinkStop(display_link: CVDisplayLinkRef) -> CVReturn;
        fn CVDisplayLinkRelease(display_link: CVDisplayLinkRef);
    }

    struct DisplayLinkState {
        link: CVDisplayLinkRef,
        is_running: bool,
    }

    // SAFETY: CVDisplayLinkRef is a thread-safe CoreVideo object.
    unsafe impl Send for DisplayLinkState {}
    // SAFETY: CVDisplayLinkRef is a thread-safe CoreVideo object.
    unsafe impl Sync for DisplayLinkState {}

    unsafe extern "C" fn display_link_callback(
        _display_link: CVDisplayLinkRef,
        _in_now: *const CVTimeStamp,
        _in_output_time: *const CVTimeStamp,
        _flags_in: CVOptionFlags,
        _flags_out: *mut CVOptionFlags,
        context: *mut c_void,
    ) -> CVReturn {
        if !context.is_null() {
            // SAFETY: context points to the live Thread in Arc<Thread> stored in DisplaySync.
            let thread = unsafe { &*(context as *const Thread) };
            thread.unpark();
        }
        0
    }

    /// macOS VSync synchronizer using `CVDisplayLink` to drive sub-pixel easing frames.
    pub struct DisplaySync {
        state: Option<DisplayLinkState>,
        _worker_thread: Arc<Thread>,
    }

    impl DisplaySync {
        /// Create a new `DisplaySync` associated with the worker thread.
        #[must_use]
        pub fn new(worker_thread: Thread) -> Self {
            let worker_thread = Arc::new(worker_thread);
            let mut link: CVDisplayLinkRef = std::ptr::null_mut();
            // SAFETY: CVDisplayLinkCreateWithActiveCGDisplays is a standard CoreVideo API.
            let ret = unsafe { CVDisplayLinkCreateWithActiveCGDisplays(&raw mut link) };
            if ret != 0 || link.is_null() {
                return Self {
                    state: None,
                    _worker_thread: worker_thread,
                };
            }

            let context_ptr = Arc::as_ptr(&worker_thread) as *mut c_void;
            // SAFETY: link is a valid CVDisplayLinkRef and context_ptr remains valid for DisplaySync lifetime.
            let cb_ret =
                unsafe { CVDisplayLinkSetOutputCallback(link, display_link_callback, context_ptr) };
            if cb_ret != 0 {
                // SAFETY: link is valid and allocated by CoreVideo.
                unsafe { CVDisplayLinkRelease(link) };
                return Self {
                    state: None,
                    _worker_thread: worker_thread,
                };
            }

            Self {
                state: Some(DisplayLinkState {
                    link,
                    is_running: false,
                }),
                _worker_thread: worker_thread,
            }
        }

        /// Start the display link if not already running.
        pub fn start(&mut self) {
            if let Some(state) = &mut self.state
                && !state.is_running
            {
                // SAFETY: link is valid.
                unsafe { CVDisplayLinkStart(state.link) };
                state.is_running = true;
            }
        }

        /// Stop the display link.
        pub fn stop(&mut self) {
            if let Some(state) = &mut self.state
                && state.is_running
            {
                // SAFETY: link is valid.
                unsafe { CVDisplayLinkStop(state.link) };
                state.is_running = false;
            }
        }
    }

    impl Drop for DisplaySync {
        fn drop(&mut self) {
            if let Some(state) = self.state.take() {
                if state.is_running {
                    // SAFETY: link is valid.
                    unsafe { CVDisplayLinkStop(state.link) };
                }
                // SAFETY: link is valid.
                unsafe { CVDisplayLinkRelease(state.link) };
            }
        }
    }
}

#[cfg(any(not(target_os = "macos"), test))]
mod imp {
    use std::thread::Thread;

    /// Non-macOS no-op display sync fallback.
    pub struct DisplaySync;

    impl DisplaySync {
        #[must_use]
        pub fn new(_worker_thread: Thread) -> Self {
            Self
        }

        #[expect(
            clippy::unused_self,
            reason = "match interface with macos platform implementation"
        )]
        pub fn start(&mut self) {}

        #[expect(
            clippy::unused_self,
            reason = "match interface with macos platform implementation"
        )]
        pub fn stop(&mut self) {}
    }
}

pub use imp::DisplaySync;
