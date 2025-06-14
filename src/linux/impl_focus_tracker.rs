use super::{utils::wayland_detect, wayland_focus_tracker, xorg_focus_tracker};
use crate::{FerrousFocusResult, FocusedWindow};
use std::sync::{Arc, atomic::AtomicBool};

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
            wayland_focus_tracker::track_focus(on_focus)
        } else {
            xorg_focus_tracker::track_focus(on_focus)
        }
    }

    pub fn track_focus_with_stop<F>(
        &self,
        on_focus: F,
        stop_signal: Arc<AtomicBool>,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        if wayland_detect() {
            wayland_focus_tracker::track_focus_with_stop(on_focus, stop_signal)
        } else {
            xorg_focus_tracker::track_focus_with_stop(on_focus, stop_signal)
        }
    }
}
