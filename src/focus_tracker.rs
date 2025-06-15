use crate::{FerrousFocusResult, FocusedWindow, platform::impl_focus_tracker::ImplFocusTracker};
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone)]
pub struct FocusTracker {
    impl_focus_tracker: ImplFocusTracker,
}

impl FocusTracker {
    pub fn new() -> Self {
        Self {
            impl_focus_tracker: ImplFocusTracker::new(),
        }
    }
}

impl Default for FocusTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusTracker {
    pub fn track_focus<F>(&self, on_focus: F) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.impl_focus_tracker.track_focus(on_focus)
    }

    pub fn track_focus_with_stop<F>(
        &self,
        on_focus: F,
        stop_signal: &AtomicBool,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.impl_focus_tracker
            .track_focus_with_stop(on_focus, stop_signal)
    }
}
