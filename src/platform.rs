use bgfx_rs::bgfx::{PlatformData, RendererType};
use glfw::Window;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub fn get_platform_data(window: &Window) -> PlatformData {
    let mut pd = PlatformData::new();

    match window.raw_window_handle() {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        RawWindowHandle::Xlib(data) => {
            pd.nwh = data.window as *mut _;
        }
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        RawWindowHandle::Wayland(data) => {
            pd.ndt = data.surface; // same as window, on wayland there ins't a concept of windows
        }

        #[cfg(target_os = "macos")]
        RawWindowHandle::AppKit(data) => {
            pd.nwh = data.ns_window;
        }
        #[cfg(target_os = "windows")]
        RawWindowHandle::Win32(data) => {
            pd.nwh = data.hwnd;
        }
        _ => panic!("Unsupported Window Manager"),
    }

    return pd;
}

#[cfg(target_os = "linux")]
pub fn get_render_type() -> RendererType {
    RendererType::OpenGL
}

#[cfg(not(target_os = "linux"))]
pub fn get_render_type() -> RendererType {
    RendererType::OpenGL
}
