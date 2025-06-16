use crate::{FerrousFocusResult, FocusedWindow, platform::impl_focus_tracker::ImplFocusTracker};
use std::sync::{atomic::AtomicBool, mpsc};

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

    /// Subscribe to focus changes and receive them via a channel
    pub fn subscribe_focus_changes(&self) -> FerrousFocusResult<mpsc::Receiver<FocusedWindow>> {
        let (sender, receiver) = mpsc::channel();
        let stop_signal = AtomicBool::new(false);

        // Clone the tracker for the background thread
        let tracker = self.clone();

        // Spawn a background thread to track focus changes
        std::thread::spawn(move || {
            let _ = tracker.track_focus_with_stop(
                move |window: FocusedWindow| -> FerrousFocusResult<()> {
                    if sender.send(window).is_err() {
                        // Receiver has been dropped, stop tracking
                        return Err(crate::FerrousFocusError::Error(
                            "Receiver dropped".to_string(),
                        ));
                    }
                    Ok(())
                },
                &stop_signal,
            );
        });

        Ok(receiver)
    }
}
