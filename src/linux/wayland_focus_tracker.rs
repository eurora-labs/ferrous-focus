use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow, IconData};
use std::process::Command;
use std::time::Duration;

pub fn track_focus<F>(mut on_focus: F) -> FerrousFocusResult<()>
where
    F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
{
    // For now, implement a basic Wayland focus tracker using swaymsg
    // This is a simplified implementation that works with Sway compositor

    // Check if we're running under Sway
    if !is_sway_available() {
        return Err(FerrousFocusError::Platform(
            "Wayland focus tracking currently only supports Sway compositor".to_string(),
        ));
    }

    let mut last_focused: Option<String> = None;

    loop {
        match get_focused_window_sway() {
            Ok(window) => {
                // Check if focus changed
                let current_title = window.window_title.clone().unwrap_or_default();
                if last_focused.as_ref() != Some(&current_title) {
                    last_focused = Some(current_title);

                    if let Err(e) = on_focus(window) {
                        eprintln!("Focus event handler failed: {}", e);
                        // Continue processing instead of propagating the error
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get focused window: {}", e);
            }
        }

        // Poll every 100ms
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn is_sway_available() -> bool {
    Command::new("swaymsg").arg("--version").output().is_ok()
}

fn get_focused_window_sway() -> FerrousFocusResult<FocusedWindow> {
    let output = Command::new("swaymsg")
        .args(&["-t", "get_tree"])
        .output()
        .map_err(|e| FerrousFocusError::Platform(format!("Failed to run swaymsg: {}", e)))?;

    if !output.status.success() {
        return Err(FerrousFocusError::Platform(
            "swaymsg command failed".to_string(),
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);

    // Parse the JSON to find the focused window
    // This is a simplified parser - in a real implementation you'd use serde_json
    if let Some(focused_window) = parse_sway_tree_for_focused(&json_str) {
        Ok(focused_window)
    } else {
        Err(FerrousFocusError::Platform(
            "No focused window found".to_string(),
        ))
    }
}

fn parse_sway_tree_for_focused(json_str: &str) -> Option<FocusedWindow> {
    // This is a very basic JSON parser for the sway tree
    // In a production implementation, you would use serde_json

    // Look for "focused": true and extract the window information
    let lines: Vec<&str> = json_str.lines().collect();
    let mut in_focused_node = false;
    let mut title: Option<String> = None;
    let mut app_id: Option<String> = None;
    let mut pid: Option<u32> = None;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("\"focused\": true") {
            in_focused_node = true;

            // Look backwards and forwards for window properties
            for j in (i.saturating_sub(20))..std::cmp::min(i + 20, lines.len()) {
                let prop_line = lines[j];

                if prop_line.contains("\"name\":") {
                    if let Some(name) = extract_json_string_value(prop_line, "name") {
                        title = Some(name);
                    }
                }

                if prop_line.contains("\"app_id\":") {
                    if let Some(id) = extract_json_string_value(prop_line, "app_id") {
                        app_id = Some(id);
                    }
                }

                if prop_line.contains("\"pid\":") {
                    if let Some(pid_str) = extract_json_number_value(prop_line, "pid") {
                        if let Ok(p) = pid_str.parse::<u32>() {
                            pid = Some(p);
                        }
                    }
                }
            }
            break;
        }
    }

    if in_focused_node && (title.is_some() || app_id.is_some()) {
        Some(FocusedWindow {
            process_id: pid,
            process_name: app_id,
            window_title: title,
            icon: Some(IconData {
                width: 0,
                height: 0,
                pixels: Vec::new(),
            }),
        })
    } else {
        None
    }
}

fn extract_json_string_value(line: &str, key: &str) -> Option<String> {
    let key_pattern = format!("\"{}\":", key);
    if let Some(start) = line.find(&key_pattern) {
        let after_key = &line[start + key_pattern.len()..];
        if let Some(quote_start) = after_key.find('"') {
            let after_quote = &after_key[quote_start + 1..];
            if let Some(quote_end) = after_quote.find('"') {
                return Some(after_quote[..quote_end].to_string());
            }
        }
    }
    None
}

fn extract_json_number_value(line: &str, key: &str) -> Option<String> {
    let key_pattern = format!("\"{}\":", key);
    if let Some(start) = line.find(&key_pattern) {
        let after_key = &line[start + key_pattern.len()..].trim();
        let mut end = 0;
        for (i, c) in after_key.char_indices() {
            if c.is_ascii_digit() {
                end = i + 1;
            } else {
                break;
            }
        }
        if end > 0 {
            return Some(after_key[..end].to_string());
        }
    }
    None
}
