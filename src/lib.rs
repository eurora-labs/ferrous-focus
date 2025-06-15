mod error;
mod focus_tracker;
mod focused_window;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;

pub use error::{FerrousFocusError, FerrousFocusResult};
pub use focus_tracker::FocusTracker;
pub use focused_window::{FocusedWindow, IconData};
