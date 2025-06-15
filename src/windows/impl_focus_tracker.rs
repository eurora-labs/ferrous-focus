use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow, IconData};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use windows_sys::Win32::Foundation::HWND;

use super::utils;

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
        // Track the previously focused window to avoid duplicate events
        let mut prev_hwnd: Option<HWND> = None;
        let mut prev_title: Option<String> = None;

        // Get initial focused window
        if let Some(hwnd) = utils::get_foreground_window() {
            if let Ok((title, process)) = utils::get_window_info(hwnd) {
                let icon = get_window_icon(hwnd).ok();

                if let Err(e) = on_focus(FocusedWindow {
                    process_id: None, // Windows doesn't easily provide PID from HWND
                    process_name: Some(process.clone()),
                    window_title: Some(title.clone()),
                    icon,
                }) {
                    eprintln!("Focus event handler failed: {}", e);
                }

                prev_hwnd = Some(hwnd);
                prev_title = Some(title);
            }
        }

        // Main event loop - we'll use polling since Windows event hooks are complex to integrate
        // with Rust's async runtime in a cross-platform way
        loop {
            // Check stop signal before processing
            if let Some(stop) = stop_signal {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
            }

            // Check current foreground window
            if let Some(current_hwnd) = utils::get_foreground_window() {
                let focus_changed = match prev_hwnd {
                    Some(prev) => prev != current_hwnd,
                    None => true,
                };

                match utils::get_window_info(current_hwnd) {
                    Ok((title, process)) => {
                        // Also check if title changed for the same window
                        let title_changed = match &prev_title {
                            Some(prev_t) => *prev_t != title,
                            None => true,
                        };

                        // Trigger handler if either window focus or title has changed
                        if focus_changed || title_changed {
                            let icon = get_window_icon(current_hwnd).ok();

                            if let Err(e) = on_focus(FocusedWindow {
                                process_id: None, // Windows doesn't easily provide PID from HWND
                                process_name: Some(process.clone()),
                                window_title: Some(title.clone()),
                                icon,
                            }) {
                                eprintln!("Focus event handler failed: {}", e);
                            }

                            prev_hwnd = Some(current_hwnd);
                            prev_title = Some(title);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get window info: {}", e);
                    }
                }
            } else {
                // No foreground window
                if prev_hwnd.is_some() {
                    prev_hwnd = None;
                    prev_title = None;
                }
            }

            // Sleep to avoid high CPU usage
            std::thread::sleep(Duration::from_millis(250));
        }

        Ok(())
    }
}

/* ------------------------------------------------------------ */
/* Helper functions                                              */
/* ------------------------------------------------------------ */

/// Get the icon for a window (simplified implementation)
fn get_window_icon(_hwnd: HWND) -> FerrousFocusResult<IconData> {
    // For now, return empty IconData as getting window icons on Windows
    // requires more complex Win32 API calls and icon extraction
    // This could be enhanced later with proper icon extraction

    // We would need to:
    // 1. Get the window's class icon or application icon
    // 2. Extract the icon data
    // 3. Convert to RGBA format
    // 4. Return as IconData

    // For the initial implementation, we'll return empty IconData
    // which matches the behavior when icon extraction fails in other platforms
    Ok(IconData {
        width: 0,
        height: 0,
        pixels: Vec::new(),
    })
}

/// Convert ARGB icon data to IconData (placeholder for future implementation)
fn _convert_icon_to_icon_data(icon_data: &[u32]) -> FerrousFocusResult<IconData> {
    if icon_data.len() < 2 {
        return Err(FerrousFocusError::Platform("Invalid icon data".to_string()));
    }

    let width = icon_data[0] as usize;
    let height = icon_data[1] as usize;

    if width == 0 || height == 0 || width > 1024 || height > 1024 {
        return Err(FerrousFocusError::Platform(
            "Invalid icon dimensions".to_string(),
        ));
    }

    // Convert ARGB to RGBA format
    let mut pixels = Vec::with_capacity(width * height * 4);

    for y in 0..height {
        for x in 0..width {
            let idx = 2 + (y * width + x);
            if idx < icon_data.len() {
                let argb = icon_data[idx];
                let a = ((argb >> 24) & 0xFF) as u8;
                let r = ((argb >> 16) & 0xFF) as u8;
                let g = ((argb >> 8) & 0xFF) as u8;
                let b = (argb & 0xFF) as u8;

                // Store as RGBA
                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(a);
            } else {
                // Fill with transparent pixels if data is missing
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Ok(IconData {
        width,
        height,
        pixels,
    })
}
