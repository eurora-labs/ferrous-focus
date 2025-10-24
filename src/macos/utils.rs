use crate::{FocusedWindow, error::FerrousFocusResult};
use base64::prelude::*;
use std::process::Command;

/// Get all information about the frontmost window in a single atomic operation.
/// Returns: (app_name, process_id, window_title)
pub fn get_frontmost_window_info() -> FerrousFocusResult<FocusedWindow> {
    #[allow(unused_unsafe)]
    unsafe {
        // Get PID and window title from AppleScript
        let window_info = get_window_info_via_applescript()?;

        // Get the localized app name from NSWorkspace using the PID
        // This gives us the user-friendly name (e.g., "Windsurf" instead of "Electron")
        // let display_name = get_localized_app_name(process_id).unwrap_or(process_name);

        Ok(window_info)
        // Ok((display_name, process_id, window_title))
    }
}

/// Get all window information via AppleScript.
/// Returns: (app_name, process_id, window_title)
unsafe fn get_window_info_via_applescript() -> FerrousFocusResult<FocusedWindow> {
    const APPLESCRIPT: &str = r#"
    use framework "Foundation"
    use framework "AppKit"
    use scripting additions

    tell application "System Events"
	set frontApp to first application process whose frontmost is true
	set frontAppName to name of frontApp
	set frontAppPID to unix id of frontApp
	set windowTitle to ""
	try
		tell frontApp to set windowTitle to name of first window
	end try
    end tell

    set nsapp to current application's NSRunningApplication's runningApplicationWithProcessIdentifier:frontAppPID
    set appURL to nsapp's bundleURL()
    set appPath to (appURL's |path|()) as text

    set ws to current application's NSWorkspace's sharedWorkspace()
    set img to ws's iconForFile:appPath
    img's setSize:{128, 128}

    set tiffData to img's TIFFRepresentation()
    set rep to current application's NSBitmapImageRep's imageRepWithData:tiffData
    set pngData to rep's representationUsingType:(current application's NSBitmapImageFileTypePNG) |properties|:(current application's NSDictionary's dictionary())
    set b64 to (pngData's base64EncodedStringWithOptions:0) as text

    set NUL to (ASCII character 0)
    return frontAppName & NUL & frontAppPID & NUL & windowTitle & NUL & b64
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
fn parse_applescript_output(bytes: &[u8]) -> FerrousFocusResult<FocusedWindow> {
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

    let icon_data = parts.next().unwrap_or_default().to_vec();
    let b64 = BASE64_STANDARD.decode(icon_data).map_err(|_| {
        crate::error::FerrousFocusError::Platform("Failed to parse icon".to_string())
    })?;
    let image = image::load_from_memory(&b64)
        .map_err(|_| crate::error::FerrousFocusError::Platform("Failed to parse icon".to_string()))?
        .to_rgba8();

    Ok(FocusedWindow {
        process_id: Some(pid),
        window_title: Some(window_title),
        process_name: Some(app_name),
        // icon: None,
        icon: Some(image),
    })
}
