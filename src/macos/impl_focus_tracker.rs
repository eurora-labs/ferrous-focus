use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow, IconData};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::info;

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
        // Track the previously focused app
        let mut prev_process: Option<String> = None;
        let mut prev_title: Option<String> = None;

        // Set up the event loop
        loop {
            // Check stop signal before processing
            if let Some(stop) = stop_signal {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
            }

            // Get the current focused window information
            match get_focused_window_info() {
                Ok((process, title, icon_data)) => {
                    // Only report focus events when the application or title changes
                    let focus_changed = match (&prev_process, &prev_title) {
                        (Some(prev_proc), Some(prev_ttl)) => {
                            *prev_proc != process || *prev_ttl != title
                        }
                        _ => true, // First run, always report
                    };

                    if focus_changed {
                        // Convert icon data to IconData if available
                        let icon = match icon_data {
                            Some(data) => match convert_icon_to_icon_data(&data) {
                                Ok(icon_data) => Some(icon_data),
                                Err(e) => {
                                    info!("Failed to convert icon data: {}", e);
                                    None
                                }
                            },
                            None => None,
                        };

                        // Create and send the focus event
                        on_focus(FocusedWindow {
                            process_id: None, // macOS doesn't easily provide PID from AppleScript
                            process_name: Some(process.clone()),
                            window_title: Some(title.clone()),
                            icon,
                        })?;

                        // Update previous values
                        prev_process = Some(process);
                        prev_title = Some(title);
                    }
                }
                Err(e) => {
                    info!("Error getting window info: {}", e);
                }
            }

            // Sleep to avoid high CPU usage (we can keep checking frequently)
            std::thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }
}

/* ------------------------------------------------------------ */
/* Helper functions                                              */
/* ------------------------------------------------------------ */

/// Get information about the currently focused window
fn get_focused_window_info() -> FerrousFocusResult<(String, String, Option<Vec<u32>>)> {
    // Get the frontmost application name using AppleScript
    let process = utils::get_frontmost_app_name().ok_or_else(|| {
        FerrousFocusError::Platform("Failed to get frontmost application name".to_string())
    })?;

    // Get the frontmost window title
    let title = utils::get_frontmost_window_title()
        .unwrap_or_else(|| format!("{} (No window title)", process));

    // Try to get the application icon
    let icon_data = get_app_icon(&process).ok();

    Ok((process, title, icon_data))
}

/// Get the application icon for a given process name
fn get_app_icon(process_name: &str) -> FerrousFocusResult<Vec<u32>> {
    // This is a simplified implementation using AppleScript to get the app icon
    // In a real implementation, we would use NSImage and other Cocoa APIs

    // Create a temporary file to save the icon
    let temp_file = format!("/tmp/app_icon_{}.png", std::process::id());

    // AppleScript to extract the application icon and save it to a file
    let script = format!(
        r#"
        try
            tell application "Finder"
                set appPath to application file "{}" as alias
                set appIcon to icon of appPath
                set tempFolder to path to temporary items as string
                set tempFile to "{}"

                tell application "System Events"
                    set iconFile to (make new file at tempFolder with properties {{name:tempFile}})
                    set iconPath to path of iconFile
                end tell

                copy appIcon to iconFile
                return POSIX path of iconPath
            end tell
        on error
            return ""
        end try
        "#,
        process_name, temp_file
    );

    // Execute the AppleScript
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| {
            FerrousFocusError::Platform(format!("Failed to execute AppleScript: {}", e))
        })?;

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

/// Convert ARGB icon data to IconData
fn convert_icon_to_icon_data(icon_data: &[u32]) -> FerrousFocusResult<IconData> {
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
