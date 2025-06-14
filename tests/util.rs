//! Common test utilities for ferrous-focus integration tests

use std::env;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Spawn a test window using the helper binary
///
/// # Arguments
/// * `title` - The window title to set
///
/// # Returns
/// A `Child` process handle for the spawned window
pub fn spawn_test_window(title: &str) -> Result<Child, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--example", "spawn_window", "--"])
        .args(&["--title", title])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd.spawn()?;

    // Give the window time to appear
    std::thread::sleep(Duration::from_millis(500));

    Ok(child)
}

/// Spawn a test window with an icon
///
/// # Arguments
/// * `title` - The window title to set
/// * `icon_path` - Path to the icon file
///
/// # Returns
/// A `Child` process handle for the spawned window
pub fn spawn_test_window_with_icon(
    title: &str,
    icon_path: &str,
) -> Result<Child, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--example", "spawn_window", "--"])
        .args(&["--title", title, "--icon", icon_path])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd.spawn()?;

    // Give the window time to appear
    std::thread::sleep(Duration::from_millis(500));

    Ok(child)
}

/// Focus a window (platform-specific implementation)
///
/// # Arguments
/// * `child` - The child process handle of the window to focus
pub fn focus_window(child: &mut Child) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        focus_window_linux(child)
    }

    #[cfg(target_os = "windows")]
    {
        focus_window_windows(child)
    }

    #[cfg(target_os = "macos")]
    {
        focus_window_macos(child)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    compile_error!("focus_window is not implemented for this platform");
}

#[cfg(target_os = "linux")]
fn focus_window_linux(child: &mut Child) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // Get the PID of the spawned window
    let pid = child.id();

    // Use wmctrl to focus the window by PID if available
    if Command::new("wmctrl").arg("-l").output().is_ok() {
        let output = Command::new("wmctrl").args(&["-l", "-p"]).output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Find the window ID for our PID
        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(window_pid) = parts[2].parse::<u32>() {
                    if window_pid == pid {
                        let window_id = parts[0];
                        // Focus the window
                        Command::new("wmctrl")
                            .args(&["-i", "-a", window_id])
                            .output()?;
                        return Ok(());
                    }
                }
            }
        }
    }

    // Fallback: use xdotool if available
    if Command::new("xdotool").arg("--version").output().is_ok() {
        Command::new("xdotool")
            .args(&["search", "--pid", &pid.to_string(), "windowactivate"])
            .output()?;

        return Ok(());
    }

    Err("Unable to focus window – neither wmctrl nor xdotool succeeded".into())
}

#[cfg(target_os = "windows")]
fn focus_window_windows(_child: &mut Child) -> Result<(), Box<dyn std::error::Error>> {
    // Windows-specific window focusing implementation
    // This would use Windows API calls to find and focus the window
    // For now, we'll implement a basic version
    Ok(())
}

#[cfg(target_os = "macos")]
fn focus_window_macos(_child: &mut Child) -> Result<(), Box<dyn std::error::Error>> {
    // macOS-specific window focusing implementation
    // This would use AppleScript or Cocoa APIs to focus the window
    // For now, we'll implement a basic version
    Ok(())
}

/// Wait for a window with the expected title to be focused
///
/// # Arguments
/// * `expected_title` - The title to wait for
/// * `timeout` - Maximum time to wait
///
/// # Returns
/// `true` if the window was focused within the timeout, `false` otherwise
pub fn wait_for_focus(expected_title: &str, timeout: Duration) -> bool {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if let Ok(focused) = get_current_focused_window() {
            if let Some(title) = focused.window_title {
                if title.contains(expected_title) {
                    return true;
                }
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    false
}

/// Get the currently focused window (for testing purposes)
fn get_current_focused_window() -> Result<ferrous_focus::FocusedWindow, Box<dyn std::error::Error>>
{
    // This is a simplified version for testing
    // In a real implementation, this would use the actual focus tracking logic

    #[cfg(target_os = "linux")]
    {
        get_focused_window_linux()
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Placeholder for other platforms
        Ok(ferrous_focus::FocusedWindow {
            process_id: None,
            process_name: Some("unknown".to_string()),
            window_title: Some("unknown".to_string()),
            icon: None,
        })
    }
}

#[cfg(target_os = "linux")]
fn get_focused_window_linux() -> Result<ferrous_focus::FocusedWindow, Box<dyn std::error::Error>> {
    use std::process::Command;

    // Try to get the focused window using xdotool
    if let Ok(output) = Command::new("xdotool")
        .args(&["getwindowfocus", "getwindowname"])
        .output()
    {
        let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(ferrous_focus::FocusedWindow {
            process_id: None,
            process_name: None,
            window_title: Some(title),
            icon: None,
        });
    }

    // Fallback
    Ok(ferrous_focus::FocusedWindow {
        process_id: None,
        process_name: Some("unknown".to_string()),
        window_title: Some("unknown".to_string()),
        icon: None,
    })
}

/// Check if integration tests should run
///
/// Tests will only run if INTEGRATION_TEST=1 environment variable is set
pub fn should_run_integration_tests() -> bool {
    env::var("INTEGRATION_TEST")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Check if we should use Wayland backend
///
/// Returns true if WAYLAND=1 environment variable is set
pub fn should_use_wayland() -> bool {
    env::var("WAYLAND").map(|v| v == "1").unwrap_or(false)
}

/// Check if we should use X11 backend
///
/// Returns true if X11=1 environment variable is set
pub fn should_use_x11() -> bool {
    env::var("X11").map(|v| v == "1").unwrap_or(false)
}

/// Setup test environment based on flags
pub fn setup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
    if !should_run_integration_tests() {
        return Err("Integration tests disabled. Set INTEGRATION_TEST=1 to enable.".into());
    }

    if should_use_wayland() {
        unsafe {
            env::set_var("WAYLAND_DISPLAY", "wayland-test");
        }
        println!("Using Wayland backend for tests");
    } else if should_use_x11() {
        unsafe {
            env::set_var("DISPLAY", ":99");
        }
        println!("Using X11 backend for tests");
    }

    Ok(())
}

/// Cleanup function to terminate child processes
pub fn cleanup_child_process(mut child: Child) -> Result<(), Box<dyn std::error::Error>> {
    // Try to terminate gracefully first
    if let Err(_) = child.kill() {
        // If kill fails, the process might have already exited
    }

    // Wait for the process to exit
    let _ = child.wait();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_flags() {
        // Test environment flag detection
        unsafe {
            env::set_var("INTEGRATION_TEST", "1");
            assert!(should_run_integration_tests());

            env::set_var("WAYLAND", "1");
            assert!(should_use_wayland());

            env::set_var("X11", "1");
            assert!(should_use_x11());

            // Cleanup
            env::remove_var("INTEGRATION_TEST");
            env::remove_var("WAYLAND");
            env::remove_var("X11");
        }
    }
}
