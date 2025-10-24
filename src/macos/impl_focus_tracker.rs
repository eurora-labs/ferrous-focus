use crate::{FerrousFocusResult, FocusedWindow};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::debug;

use super::utils;

/// Polling interval for focus change detection (in milliseconds).
const POLL_INTERVAL_MS: u64 = 200;

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
        self.run(on_focus, None)
    }

    pub fn track_focus_with_stop<F>(
        &self,
        on_focus: F,
        stop_signal: &AtomicBool,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.run(on_focus, Some(stop_signal))
    }

    fn run<F>(&self, mut on_focus: F, stop_signal: Option<&AtomicBool>) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        let mut prev_state: Option<(Option<String>, Option<String>)> = None;

        loop {
            // Check stop signal before processing
            if should_stop(stop_signal) {
                debug!("Stop signal received, exiting focus tracking loop");
                break;
            }

            // Get the current focused window information
            match get_focused_window_info() {
                Ok(window) => {
                    let current_state = (window.process_name.clone(), window.window_title.clone());

                    // Only report focus events when the application or title changes
                    if prev_state.as_ref() != Some(&current_state) {
                        on_focus(window)?;

                        prev_state = Some(current_state);
                    }
                }
                Err(e) => {
                    debug!("Error getting window info: {}", e);
                }
            }

            std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        }

        Ok(())
    }
}

/* ------------------------------------------------------------ */
/* Helper functions                                              */
/* ------------------------------------------------------------ */

/// Check if the stop signal is set.
fn should_stop(stop_signal: Option<&AtomicBool>) -> bool {
    stop_signal.is_some_and(|stop| stop.load(Ordering::Relaxed))
}

/// Get information about the currently focused window.
fn get_focused_window_info() -> FerrousFocusResult<FocusedWindow> {
    utils::get_frontmost_window_info()
}
