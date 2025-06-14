use crate::{FerrousFocusResult, FocusedWindow, platform::impl_focus_tracker::ImplFocusTracker};
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

impl FocusTracker {
    pub fn track_focus<F>(&self, on_focus: F) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.impl_focus_tracker.track_focus(on_focus)
    }
}
