use super::{utils::wayland_detect, xorg_focus_tracker};
use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow};
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone)]
pub struct ImplFocusTracker {}

impl ImplFocusTracker {
    pub fn new() -> Self {
        Self {}
    }
}

impl ImplFocusTracker {
    pub fn track_focus<F>(&self, on_focus: F) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        if wayland_detect() {
            // Wayland is not supported for the time being
            Err(FerrousFocusError::Unsupported)
        } else {
            xorg_focus_tracker::track_focus(on_focus)
        }
    }

    pub fn track_focus_with_stop<F>(
        &self,
        on_focus: F,
        stop_signal: &AtomicBool,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        if wayland_detect() {
            // Wayland is not supported for the time being
            Err(FerrousFocusError::Unsupported)
        } else {
            xorg_focus_tracker::track_focus_with_stop(on_focus, stop_signal)
        }
    }
}
