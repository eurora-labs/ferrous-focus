use crate::{FerrousFocusResult, FocusTrackerConfig, FocusedWindow};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

#[cfg(feature = "async")]
use std::future::Future;

use super::utils;

#[derive(Debug, Clone)]
pub(crate) struct ImplFocusTracker {}

impl ImplFocusTracker {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl ImplFocusTracker {
    pub fn track_focus<F>(&self, on_focus: F, config: &FocusTrackerConfig) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.run(on_focus, None, config)
    }

    pub fn track_focus_with_stop<F>(
        &self,
        on_focus: F,
        stop_signal: &AtomicBool,
        config: &FocusTrackerConfig,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.run(on_focus, Some(stop_signal), config)
    }

    #[cfg(feature = "async")]
    pub async fn track_focus_async<F, Fut>(
        &self,
        on_focus: F,
        config: &FocusTrackerConfig,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> Fut,
        Fut: Future<Output = FerrousFocusResult<()>>,
    {
        self.run_async(on_focus, None, config).await
    }

    #[cfg(feature = "async")]
    pub async fn track_focus_async_with_stop<F, Fut>(
        &self,
        on_focus: F,
        stop_signal: &AtomicBool,
        config: &FocusTrackerConfig,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> Fut,
        Fut: Future<Output = FerrousFocusResult<()>>,
    {
        self.run_async(on_focus, Some(stop_signal), config).await
    }

    #[cfg(feature = "async")]
    async fn run_async<F, Fut>(
        &self,
        mut on_focus: F,
        stop_signal: Option<&AtomicBool>,
        config: &FocusTrackerConfig,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> Fut,
        Fut: Future<Output = FerrousFocusResult<()>>,
    {
        let mut prev_state: Option<(Option<String>, Option<String>)> = None;

        loop {
            // Check stop signal before processing
            if should_stop(stop_signal) {
                debug!("Stop signal received, exiting focus tracking loop");
                break;
            }

            // Get the current focused window information
            match get_focused_window_info(&config.icon) {
                Ok(window) => {
                    let current_state = (window.process_name.clone(), window.window_title.clone());

                    // Only report focus events when the application or title changes
                    if prev_state.as_ref() != Some(&current_state) {
                        on_focus(window).await?;

                        prev_state = Some(current_state);
                    }
                }
                Err(e) => {
                    debug!("Error getting window info: {}", e);
                }
            }

            tokio::time::sleep(config.poll_interval).await;
        }

        Ok(())
    }

    fn run<F>(
        &self,
        mut on_focus: F,
        stop_signal: Option<&AtomicBool>,
        config: &FocusTrackerConfig,
    ) -> FerrousFocusResult<()>
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
            match get_focused_window_info(&config.icon) {
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

            std::thread::sleep(config.poll_interval);
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
fn get_focused_window_info(
    icon_config: &crate::config::IconConfig,
) -> FerrousFocusResult<FocusedWindow> {
    utils::get_frontmost_window_info(icon_config)
}
