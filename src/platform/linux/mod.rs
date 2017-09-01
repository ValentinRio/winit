#![cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "openbsd"))]

use std::collections::VecDeque;
use std::sync::Arc;
use std::env;

use {CreationError, CursorState, EventsLoopClosed, MouseCursor, ControlFlow, FullScreenState};
use libc;

use self::x11::XConnection;
use self::x11::XError;
use self::x11::ffi::XVisualInfo;

pub use self::x11::XNotSupported;

mod dlopen;
pub mod wayland;
pub mod x11;

/// Environment variable specifying which backend should be used on unix platform.
///
/// Legal values are x11 and wayland. If this variable is set only the named backend
/// will be tried by winit. If it is not set, winit will try to connect to a wayland connection,
/// and if it fails will fallback on x11.
///
/// If this variable is set with any other value, winit will panic.
const BACKEND_PREFERENCE_ENV_VAR: &str = "WINIT_UNIX_BACKEND";

#[derive(Clone, Default)]
pub struct PlatformSpecificWindowBuilderAttributes {
    pub visual_infos: Option<XVisualInfo>,
    pub screen_id: Option<i32>,
}

lazy_static!(
    pub static ref X11_BACKEND: Result<Arc<XConnection>, XNotSupported> = {
        XConnection::new(Some(x_error_callback)).map(Arc::new)
    };
);

pub enum Window2 {
    X(x11::Window2),
    Wayland(wayland::Window)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowId {
    X(x11::WindowId),
    Wayland(wayland::WindowId)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceId {
    X(x11::DeviceId),
    Wayland(wayland::DeviceId)
}

#[derive(Clone)]
pub enum MonitorId {
    X(x11::MonitorId),
    Wayland(wayland::MonitorId),
    None,
}

impl MonitorId {
    #[inline]
    pub fn get_name(&self) -> Option<String> {
        match self {
            &MonitorId::X(ref m) => m.get_name(),
            &MonitorId::Wayland(ref m) => m.get_name(),
            &MonitorId::None => None,
        }
    }

    #[inline]
    pub fn get_native_identifier(&self) -> u32 {
        match self {
            &MonitorId::X(ref m) => m.get_native_identifier(),
            &MonitorId::Wayland(ref m) => m.get_native_identifier(),
            &MonitorId::None => unimplemented!()        // FIXME:
        }
    }

    #[inline]
    pub fn get_dimensions(&self) -> (u32, u32) {
        match self {
            &MonitorId::X(ref m) => m.get_dimensions(),
            &MonitorId::Wayland(ref m) => m.get_dimensions(),
            &MonitorId::None => (800, 600),     // FIXME:
        }
    }
}

impl Window2 {
    #[inline]
    pub fn new(events_loop: &EventsLoop,
               window: &::WindowAttributes,
               pl_attribs: &PlatformSpecificWindowBuilderAttributes)
               -> Result<Self, CreationError>
    {
        match *events_loop {
            EventsLoop::Wayland(ref evlp) => {
                wayland::Window::new(evlp, window).map(Window2::Wayland)
            },

            EventsLoop::X(ref el) => {
                x11::Window2::new(el, window, pl_attribs).map(Window2::X)
            },
        }
    }

    #[inline]
    pub fn id(&self) -> WindowId {
        match self {
            &Window2::X(ref w) => WindowId::X(w.id()),
            &Window2::Wayland(ref w) => WindowId::Wayland(w.id())
        }
    }

    #[inline]
    pub fn set_title(&self, title: &str) {
        match self {
            &Window2::X(ref w) => w.set_title(title),
            &Window2::Wayland(ref w) => w.set_title(title)
        }
    }

    #[inline]
    pub fn show(&self) {
        match self {
            &Window2::X(ref w) => w.show(),
            &Window2::Wayland(ref w) => w.show()
        }
    }

    #[inline]
    pub fn hide(&self) {
        match self {
            &Window2::X(ref w) => w.hide(),
            &Window2::Wayland(ref w) => w.hide()
        }
    }

    #[inline]
    pub fn get_position(&self) -> Option<(i32, i32)> {
        match self {
            &Window2::X(ref w) => w.get_position(),
            &Window2::Wayland(ref w) => w.get_position()
        }
    }

    #[inline]
    pub fn set_position(&self, x: i32, y: i32) {
        match self {
            &Window2::X(ref w) => w.set_position(x, y),
            &Window2::Wayland(ref w) => w.set_position(x, y)
        }
    }

    #[inline]
    pub fn get_inner_size(&self) -> Option<(u32, u32)> {
        match self {
            &Window2::X(ref w) => w.get_inner_size(),
            &Window2::Wayland(ref w) => w.get_inner_size()
        }
    }

    #[inline]
    pub fn get_outer_size(&self) -> Option<(u32, u32)> {
        match self {
            &Window2::X(ref w) => w.get_outer_size(),
            &Window2::Wayland(ref w) => w.get_outer_size()
        }
    }

    #[inline]
    pub fn set_inner_size(&self, x: u32, y: u32) {
        match self {
            &Window2::X(ref w) => w.set_inner_size(x, y),
            &Window2::Wayland(ref w) => w.set_inner_size(x, y)
        }
    }

    #[inline]
    pub fn set_cursor(&self, cursor: MouseCursor) {
        match self {
            &Window2::X(ref w) => w.set_cursor(cursor),
            &Window2::Wayland(ref w) => w.set_cursor(cursor)
        }
    }

    #[inline]
    pub fn set_cursor_state(&self, state: CursorState) -> Result<(), String> {
        match self {
            &Window2::X(ref w) => w.set_cursor_state(state),
            &Window2::Wayland(ref w) => w.set_cursor_state(state)
        }
    }

    #[inline]
    pub fn hidpi_factor(&self) -> f32 {
       match self {
            &Window2::X(ref w) => w.hidpi_factor(),
            &Window2::Wayland(ref w) => w.hidpi_factor()
        }
    }

    #[inline]
    pub fn set_cursor_position(&self, x: i32, y: i32) -> Result<(), ()> {
        match self {
            &Window2::X(ref w) => w.set_cursor_position(x, y),
            &Window2::Wayland(ref w) => w.set_cursor_position(x, y)
        }
    }

    #[inline]
    pub fn platform_display(&self) -> *mut libc::c_void {
        use wayland_client::Proxy;
        match self {
            &Window2::X(ref w) => w.platform_display(),
            &Window2::Wayland(ref w) => w.get_display().ptr() as *mut _
        }
    }

    #[inline]
    pub fn platform_window(&self) -> *mut libc::c_void {
        use wayland_client::Proxy;
        match self {
            &Window2::X(ref w) => w.platform_window(),
            &Window2::Wayland(ref w) => w.get_surface().ptr() as *mut _
        }
    }

    #[inline]
    pub fn set_maximized(&self, maximized: bool) {
        match self {
            &Window2::X(ref w) => w.set_maximized(maximized),
            &Window2::Wayland(ref _w) => {},
        }
    }

    #[inline]
    pub fn set_fullscreen(&self, state: FullScreenState) {
        match self {
            &Window2::X(ref w) => w.set_fullscreen(state),
            &Window2::Wayland(ref _w) => {},
        }
    }
}

unsafe extern "C" fn x_error_callback(dpy: *mut x11::ffi::Display, event: *mut x11::ffi::XErrorEvent)
                                      -> libc::c_int
{
    use std::ffi::CStr;

    if let Ok(ref x) = *X11_BACKEND {
        let mut buff: Vec<u8> = Vec::with_capacity(1024);
        (x.xlib.XGetErrorText)(dpy, (*event).error_code as i32, buff.as_mut_ptr() as *mut libc::c_char, buff.capacity() as i32);
        let description = CStr::from_ptr(buff.as_mut_ptr() as *const libc::c_char).to_string_lossy();

        let error = XError {
            description: description.into_owned(),
            error_code: (*event).error_code,
            request_code: (*event).request_code,
            minor_code: (*event).minor_code,
        };

        *x.latest_error.lock().unwrap() = Some(error);
    }

    0
}

pub enum EventsLoop {
    Wayland(wayland::EventsLoop),
    X(x11::EventsLoop)
}

pub enum EventsLoopProxy {
    X(x11::EventsLoopProxy),
    Wayland(wayland::EventsLoopProxy),
}

impl EventsLoop {
    pub fn new() -> EventsLoop {
        if let Ok(env_var) = env::var(BACKEND_PREFERENCE_ENV_VAR) {
            match env_var.as_str() {
                "x11" => {
                    return EventsLoop::new_x11().unwrap();      // TODO: propagate
                },
                "wayland" => {
                    match EventsLoop::new_wayland() {
                        Ok(e) => return e,
                        Err(_) => panic!()      // TODO: propagate
                    }
                },
                _ => panic!("Unknown environment variable value for {}, try one of `x11`,`wayland`",
                            BACKEND_PREFERENCE_ENV_VAR),
            }
        }

        if let Ok(el) = EventsLoop::new_wayland() {
            return el;
        }

        if let Ok(el) = EventsLoop::new_x11() {
            return el;
        }

        panic!("No backend is available")
    }

    pub fn new_wayland() -> Result<EventsLoop, ()> {
        wayland::WaylandContext::init()
            .map(|ctx| EventsLoop::Wayland(wayland::EventsLoop::new(Arc::new(ctx))))
            .ok_or(())
    }

    pub fn new_x11() -> Result<EventsLoop, XNotSupported> {
        match *X11_BACKEND {
            Ok(ref x) => Ok(EventsLoop::X(x11::EventsLoop::new(x.clone()))),
            Err(ref err) => Err(err.clone()),
        }
    }

    #[inline]
    pub fn get_available_monitors(&self) -> VecDeque<MonitorId> {
        match *self {
            EventsLoop::Wayland(ref evlp) => wayland::get_available_monitors(evlp.context())
                                    .into_iter()
                                    .map(MonitorId::Wayland)
                                    .collect(),
            EventsLoop::X(ref evlp) => x11::get_available_monitors(evlp.x_connection())
                                        .into_iter()
                                        .map(MonitorId::X)
                                        .collect(),
        }
    }

    #[inline]
    pub fn get_primary_monitor(&self) -> MonitorId {
        match *self {
            EventsLoop::Wayland(ref evlp) => MonitorId::Wayland(wayland::get_primary_monitor(evlp.context())),
            EventsLoop::X(ref evlp) => MonitorId::X(x11::get_primary_monitor(evlp.x_connection())),
        }
    }

    pub fn create_proxy(&self) -> EventsLoopProxy {
        match *self {
            EventsLoop::Wayland(ref evlp) => EventsLoopProxy::Wayland(evlp.create_proxy()),
            EventsLoop::X(ref evlp) => EventsLoopProxy::X(evlp.create_proxy()),
        }
    }

    pub fn poll_events<F>(&mut self, callback: F)
        where F: FnMut(::Event)
    {
        match *self {
            EventsLoop::Wayland(ref mut evlp) => evlp.poll_events(callback),
            EventsLoop::X(ref mut evlp) => evlp.poll_events(callback)
        }
    }

    pub fn run_forever<F>(&mut self, callback: F)
        where F: FnMut(::Event) -> ControlFlow
    {
        match *self {
            EventsLoop::Wayland(ref mut evlp) => evlp.run_forever(callback),
            EventsLoop::X(ref mut evlp) => evlp.run_forever(callback)
        }
    }
}

impl EventsLoopProxy {
    pub fn wakeup(&self) -> Result<(), EventsLoopClosed> {
        match *self {
            EventsLoopProxy::Wayland(ref proxy) => proxy.wakeup(),
            EventsLoopProxy::X(ref proxy) => proxy.wakeup(),
        }
    }
}
