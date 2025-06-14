mod error;
mod window;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
pub mod platform;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
pub mod platform;
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub mod platform;

pub use error::{FerrousFocusError, FerrousFocusResult};
pub use window::{FocusedWindow, IconData};

/// Start tracking focus changes
///
/// # Arguments
/// * `on_focus` - Callback function that will be called when focus changes
///
/// # Returns
/// Result indicating success or failure of the focus tracking setup
pub fn track_focus<F>(on_focus: F) -> FerrousFocusResult<()>
where
    F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
{
    #[cfg(target_os = "linux")]
    {
        let tracker = platform::impl_focus_tracker::ImplFocusTracker::new();
        tracker.track_focus(on_focus)
    }

    #[cfg(target_os = "windows")]
    {
        // Windows implementation would go here
        Err(FerrousFocusError::Unsupported)
    }

    #[cfg(target_os = "macos")]
    {
        // macOS implementation would go here
        Err(FerrousFocusError::Unsupported)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(FerrousFocusError::Unsupported)
    }
}
