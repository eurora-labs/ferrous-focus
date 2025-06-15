use crate::error::FerrousFocusResult;
use objc2_app_kit::NSWorkspace;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

// Cache structure for reducing repeated API calls
#[derive(Debug, Clone)]
struct CachedResult {
    value: String,
    timestamp: Instant,
}

static APP_NAME_CACHE: OnceLock<Mutex<Option<CachedResult>>> = OnceLock::new();
static WINDOW_TITLE_CACHE: OnceLock<Mutex<Option<CachedResult>>> = OnceLock::new();

const CACHE_DURATION: Duration = Duration::from_millis(500); // 500ms cache for more responsive detection

/// Get the name of the frontmost application using objc2 APIs
pub fn get_frontmost_app_name() -> FerrousFocusResult<Option<String>> {
    // Check cache first
    let cache = APP_NAME_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(cached) = cache.lock() {
        if let Some(ref result) = *cached {
            if result.timestamp.elapsed() < CACHE_DURATION {
                return Ok(Some(result.value.clone()));
            }
        }
    }

    unsafe {
        // Get shared workspace
        let workspace = NSWorkspace::sharedWorkspace();
        let frontmost_app = workspace.frontmostApplication();

        if let Some(app) = frontmost_app {
            if let Some(app_name) = app.localizedName() {
                let name_str = app_name.to_string();

                // Update cache
                if let Ok(mut cached) = cache.lock() {
                    *cached = Some(CachedResult {
                        value: name_str.clone(),
                        timestamp: Instant::now(),
                    });
                }

                return Ok(Some(name_str));
            }
        }

        Ok(None)
    }
}

/// Get the title of the frontmost window using objc2 APIs
pub fn get_frontmost_window_title() -> FerrousFocusResult<Option<String>> {
    // Check cache first
    let cache = WINDOW_TITLE_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(cached) = cache.lock() {
        if let Some(ref result) = *cached {
            if result.timestamp.elapsed() < CACHE_DURATION {
                return Ok(Some(result.value.clone()));
            }
        }
    }

    unsafe {
        // Get shared workspace
        let workspace = NSWorkspace::sharedWorkspace();
        let frontmost_app = workspace.frontmostApplication();

        if let Some(_app) = frontmost_app {
            // Try to get the window title using AppleScript as a fallback
            // This is more reliable than accessibility APIs for basic window titles
            let title = get_window_title_via_applescript().unwrap_or(None);

            if let Some(ref title_str) = title {
                // Update cache
                if let Ok(mut cached) = cache.lock() {
                    *cached = Some(CachedResult {
                        value: title_str.clone(),
                        timestamp: Instant::now(),
                    });
                }
            }

            return Ok(title);
        }

        Ok(None)
    }
}

/// Helper function to get window title via AppleScript
unsafe fn get_window_title_via_applescript() -> FerrousFocusResult<Option<String>> {
    use std::process::Command;

    let script = r#"
    tell application "System Events"
        set frontApp to first application process whose frontmost is true
        set frontAppName to name of frontApp

        tell process frontAppName
            try
                set windowTitle to name of first window
                return windowTitle
            on error
                return ""
            end try
        end tell
    end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| {
            crate::error::FerrousFocusError::Platform(format!(
                "Failed to execute AppleScript: {}",
                e
            ))
        })?;

    if output.status.success() {
        let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !title.is_empty() {
            return Ok(Some(title));
        }
    } else {
        // Check if the error is related to accessibility permissions
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not allowed assistive access")
            || stderr.contains("accessibility")
            || stderr.contains("permission")
        {
            return Err(crate::error::FerrousFocusError::PermissionDenied);
        }
    }

    Ok(None)
}

/// Check if the application is running in a sandboxed environment
pub fn is_app_sandboxed() -> bool {
    // Check for the App Sandbox environment variable
    std::env::var("APP_SANDBOX_CONTAINER_ID").is_ok()
}

/// Get the bundle identifier for a given application name using objc2 APIs
pub fn get_bundle_id_for_app(app_name: &str) -> FerrousFocusResult<Option<String>> {
    unsafe {
        // Get shared workspace
        let workspace = NSWorkspace::sharedWorkspace();
        let running_apps = workspace.runningApplications();

        for app in running_apps.iter() {
            if let Some(localized_name) = app.localizedName() {
                let name_str = localized_name.to_string();
                if name_str == app_name {
                    if let Some(bundle_id) = app.bundleIdentifier() {
                        return Ok(Some(bundle_id.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }
}

/// Clear all caches - useful for testing or when you need fresh data
pub fn clear_caches() {
    if let Some(cache) = APP_NAME_CACHE.get() {
        if let Ok(mut cached) = cache.lock() {
            *cached = None;
        }
    }

    if let Some(cache) = WINDOW_TITLE_CACHE.get() {
        if let Ok(mut cached) = cache.lock() {
            *cached = None;
        }
    }
}
