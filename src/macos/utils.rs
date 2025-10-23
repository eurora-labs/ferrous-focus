use crate::error::FerrousFocusResult;
use objc2_app_kit::NSWorkspace;
use std::process::Command;

/// Get all information about the frontmost window in a single atomic operation.
/// Returns: (app_name, process_id, window_title)
pub fn get_frontmost_window_info() -> FerrousFocusResult<(String, u32, String)> {
    unsafe {
        // Get PID and window title from AppleScript
        let (process_name, process_id, window_title) = get_window_info_via_applescript()?;

        // Get the localized app name from NSWorkspace using the PID
        // This gives us the user-friendly name (e.g., "Windsurf" instead of "Electron")
        let display_name = get_localized_app_name(process_id).unwrap_or(process_name);

        Ok((display_name, process_id, window_title))
    }
}

/// Get the localized application name for a given process ID.
fn get_localized_app_name(process_id: u32) -> Option<String> {
    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let running_apps = workspace.runningApplications();

        running_apps
            .iter()
            .find(|app| app.processIdentifier() as u32 == process_id)
            .and_then(|app| app.localizedName().map(|name| name.to_string()))
    }
}

/// Get all window information via AppleScript.
/// Returns: (app_name, process_id, window_title)
unsafe fn get_window_info_via_applescript() -> FerrousFocusResult<(String, u32, String)> {
    const APPLESCRIPT: &str = r#"
    tell application "System Events"
        set frontApp to first application process whose frontmost is true
        set frontAppName to name of frontApp
        set frontAppPID to unix id of frontApp

        tell process frontAppName
            try
                set windowTitle to name of first window
            on error
                set windowTitle to ""
            end try
        end tell
        set NUL to (ASCII character 0)
        return frontAppName & NUL & frontAppPID & NUL & windowTitle
    end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(APPLESCRIPT)
        .output()
        .map_err(|e| {
            crate::error::FerrousFocusError::Platform(format!(
                "Failed to execute AppleScript: {}",
                e
            ))
        })?;

    if !output.status.success() {
        // Check if the error is related to accessibility permissions
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not allowed assistive access")
            || stderr.contains("accessibility")
            || stderr.contains("permission")
        {
            return Err(crate::error::FerrousFocusError::PermissionDenied);
        }

        return Err(crate::error::FerrousFocusError::Platform(
            "Failed to get window info via AppleScript".to_string(),
        ));
    }
    let mut bytes = output.stdout;
    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }

    parse_applescript_output(&bytes)
}

/// Parse the AppleScript output into structured data.
fn parse_applescript_output(bytes: &[u8]) -> FerrousFocusResult<(String, u32, String)> {
    let mut parts = bytes.split(|&b| b == 0);
    let app_name = String::from_utf8(parts.next().unwrap_or_default().to_vec()).map_err(|_| {
        crate::error::FerrousFocusError::Platform("Failed to parse app name".to_string())
    })?;
    let pid: u32 = String::from_utf8(parts.next().unwrap_or_default().to_vec())
        .map_err(|_| {
            crate::error::FerrousFocusError::Platform("Failed to parse process ID".to_string())
        })?
        .parse()
        .map_err(|_| {
            crate::error::FerrousFocusError::Platform("Failed to parse process ID".to_string())
        })?;
    let window_title =
        String::from_utf8(parts.next().unwrap_or_default().to_vec()).map_err(|_| {
            crate::error::FerrousFocusError::Platform("Failed to parse window title".to_string())
        })?;

    Ok((app_name, pid, window_title))
}
