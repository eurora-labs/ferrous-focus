use super::{utils::wayland_detect, wayland_focus_tracker, xorg_focus_tracker};
use crate::{FerrousFocusResult, FocusedWindow};

#[derive(Debug, Clone)]
pub(crate) struct ImplFocusTracker {}

impl ImplFocusTracker {
    pub(crate) fn new() -> Self {
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
}
