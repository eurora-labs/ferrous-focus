use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow, RgbaImage};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{debug, info};

use super::utils;

/// Polling interval for focus change detection (in milliseconds).
const POLL_INTERVAL_MS: u64 = 200;

/// Window information tuple: (process_name, window_title, process_id, icon_data)
type WindowInfo = (String, String, Option<u32>, Option<Vec<u32>>);

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
        let mut prev_state: Option<(String, String)> = None;

        loop {
            // Check stop signal before processing
            if should_stop(stop_signal) {
                debug!("Stop signal received, exiting focus tracking loop");
                break;
            }

            // Get the current focused window information
            match get_focused_window_info() {
                Ok((process, title, process_id, icon_data)) => {
                    let current_state = (process.clone(), title.clone());

                    // Only report focus events when the application or title changes
                    if prev_state.as_ref() != Some(&current_state) {
                        info!("Focus changed: {} - {}", process, title);

                        // Convert icon data to RgbaImage if available
                        let icon = match icon_data {
                            Some(data) => match convert_icon_to_rgba_image(&data) {
                                Ok(icon_data) => Some(icon_data),
                                Err(e) => {
                                    info!("Failed to convert icon data: {}", e);
                                    None
                                }
                            },
                            None => None,
                        };

                        on_focus(FocusedWindow {
                            process_id,
                            process_name: Some(process),
                            window_title: Some(title),
                            icon,
                        })?;

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
fn get_focused_window_info() -> FerrousFocusResult<WindowInfo> {
    let (process, process_id, title) = utils::get_frontmost_window_info()?;
    let icon_data = get_app_icon(&process).ok();
    Ok((process, title, Some(process_id), icon_data))
}

/// Get the application icon for a given process name.
///
/// This is a placeholder implementation that returns minimal icon metadata.
/// A full implementation would require NSImage manipulation and image processing libraries.
fn get_app_icon(_process_name: &str) -> FerrousFocusResult<Vec<u32>> {
    // Return placeholder icon data (width=32, height=32, no pixel data)
    Ok(vec![32, 32])
}

/// Get the application icon for a given process name
fn get_app_icon(process_name: &str) -> FerrousFocusResult<Vec<u32>> {
    // This is a simplified implementation using AppleScript to get the app icon
    // In a real implementation, we would use NSImage and other Cocoa APIs

    // Create a temporary file to save the icon
    let temp_file = format!("/tmp/app_icon_{}.png", std::process::id());

    // AppleScript to extract the application icon and save it to a file
    // Use AppleScript's 'quoted form of' to safely handle process names with special characters
    let escaped_process_name = shell_escape_for_applescript(process_name);
    let escaped_temp_file = shell_escape_for_applescript(&temp_file);

    let script = format!(
        r#"
        try
            set processName to {escaped_process_name}
            set tempFileName to {escaped_temp_file}
            tell application "Finder"
                set appPath to application file processName as alias
                set appIcon to icon of appPath
                set tempFolder to path to temporary items as string

                tell application "System Events"
                    set iconFile to (make new file at tempFolder with properties {{name:tempFileName}})
                    set iconPath to path of iconFile
                end tell

                copy appIcon to iconFile
                return POSIX path of iconPath
            end tell
        on error
            return ""
        end try
        "#
    );

    // Execute the AppleScript
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| FerrousFocusError::Platform(format!("Failed to execute AppleScript: {e}")))?;

    if !output.status.success() {
        return Err(FerrousFocusError::Platform(
            "AppleScript execution failed".to_string(),
        ));
    }

    // Check if the icon file was created
    let icon_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if icon_path.is_empty() {
        return Err(FerrousFocusError::Platform(
            "Failed to get application icon".to_string(),
        ));
    }

    // For now, return a simple placeholder since we're removing image dependency
    // In a real implementation, we would load and process the icon file
    let _ = std::fs::remove_file(icon_path);

    // Return placeholder icon data (width, height, then ARGB pixels)
    Ok(vec![32, 32]) // Just width and height, no actual pixel data
}

/// Convert ARGB icon data to RgbaImage
fn convert_icon_to_rgba_image(icon_data: &[u32]) -> FerrousFocusResult<RgbaImage> {
    if icon_data.len() < 2 {
        return Err(FerrousFocusError::Platform(
            "Invalid icon data: insufficient length".to_string(),
        ));
    }

    let width = icon_data[0] as usize;
    let height = icon_data[1] as usize;

    validate_icon_dimensions(width, height)?;

    let pixels = convert_argb_to_rgba(icon_data, width, height);

    // Create RgbaImage from the pixel data
    RgbaImage::from_raw(width as u32, height as u32, pixels)
        .ok_or_else(|| FerrousFocusError::Platform("Failed to create RgbaImage".into()))
}

/// Validate icon dimensions are within acceptable bounds.
fn validate_icon_dimensions(width: usize, height: usize) -> FerrousFocusResult<()> {
    const MAX_DIMENSION: usize = 1024;

    if width == 0 || height == 0 {
        return Err(FerrousFocusError::Platform(
            "Invalid icon dimensions: zero size".to_string(),
        ));
    }

    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(FerrousFocusError::Platform(format!(
            "Invalid icon dimensions: {}x{} exceeds maximum {}x{}",
            width, height, MAX_DIMENSION, MAX_DIMENSION
        )));
    }

    Ok(())
}

/// Convert ARGB pixel data to RGBA format.
fn convert_argb_to_rgba(icon_data: &[u32], width: usize, height: usize) -> Vec<u8> {
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

                pixels.extend_from_slice(&[r, g, b, a]);
            } else {
                // Fill with transparent pixels if data is missing
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    pixels
}
