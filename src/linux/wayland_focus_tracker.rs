use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow, IconData};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use swayipc::{Connection, Event, EventType, WindowChange};

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

    // Outer loop for connection management and reconnection
    loop {
        // Connect to swayipc and subscribe to window events
        let connection = Connection::new().map_err(|e| {
            FerrousFocusError::Platform(format!("Failed to connect to sway IPC: {}", e))
        })?;

        let event_iterator = connection.subscribe([EventType::Window]).map_err(|e| {
            FerrousFocusError::Platform(format!("Failed to subscribe to window events: {}", e))
        })?;

        // Process events as they arrive
        let mut should_reconnect = false;
        for event in event_iterator {
            match event {
                Ok(Event::Window(window_event)) => {
                    // Only handle focus events
                    if matches!(window_event.change, WindowChange::Focus) {
                        match get_focused_window_from_event(&window_event) {
                            Ok(window) => {
                                // Check if focus actually changed
                                let current_title = window.window_title.clone().unwrap_or_default();
                                if last_focused.as_ref() != Some(&current_title) {
                                    last_focused = Some(current_title);

                                    on_focus(window)?;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to get focused window from event: {}", e);
                            }
                        }
                    }
                }
                Ok(_) => {
                    // Ignore other event types
                }
                Err(e) => {
                    eprintln!("Error receiving window event: {}", e);
                    eprintln!("Attempting to reconnect to sway IPC...");
                    should_reconnect = true;
                    break;
                }
            }
        }

        // If we need to reconnect, continue the outer loop to recreate connection and iterator
        if should_reconnect {
            continue;
        }

        // If we reach here without needing to reconnect, we're done
        break;
    }

    Ok(())
}

pub fn track_focus_with_stop<F>(mut on_focus: F, stop_signal: &AtomicBool) -> FerrousFocusResult<()>
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

    // Outer loop for connection management and reconnection
    loop {
        // Check stop signal before attempting connection
        if stop_signal.load(Ordering::Relaxed) {
            break;
        }

        // Connect to swayipc and subscribe to window events
        let connection = Connection::new().map_err(|e| {
            FerrousFocusError::Platform(format!("Failed to connect to sway IPC: {}", e))
        })?;

        let event_iterator = connection.subscribe([EventType::Window]).map_err(|e| {
            FerrousFocusError::Platform(format!("Failed to subscribe to window events: {}", e))
        })?;

        // Process events as they arrive
        let mut should_reconnect = false;
        for event in event_iterator {
            // Check stop signal before processing each event
            if stop_signal.load(Ordering::Relaxed) {
                return Ok(());
            }

            match event {
                Ok(Event::Window(window_event)) => {
                    // Only handle focus events
                    if matches!(window_event.change, WindowChange::Focus) {
                        match get_focused_window_from_event(&window_event) {
                            Ok(window) => {
                                // Check if focus actually changed
                                let current_title = window.window_title.clone().unwrap_or_default();
                                if last_focused.as_ref() != Some(&current_title) {
                                    last_focused = Some(current_title);

                                    on_focus(window)?;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to get focused window from event: {}", e);
                            }
                        }
                    }
                }
                Ok(_) => {
                    // Ignore other event types
                }
                Err(e) => {
                    eprintln!("Error receiving window event: {}", e);
                    eprintln!("Attempting to reconnect to sway IPC...");
                    should_reconnect = true;
                    break;
                }
            }
        }

        // If we need to reconnect, continue the outer loop to recreate connection and iterator
        if should_reconnect {
            continue;
        }

        // If we reach here without needing to reconnect, we're done
        break;
    }

    Ok(())
}

fn is_sway_available() -> bool {
    Command::new("swaymsg").arg("--version").output().is_ok()
}

fn get_focused_window_from_event(
    window_event: &swayipc::WindowEvent,
) -> FerrousFocusResult<FocusedWindow> {
    let container = &window_event.container;

    let window_title = container.name.clone();
    let process_name = container.app_id.clone();
    let process_id = container.pid.map(|p| p as u32);

    Ok(FocusedWindow {
        process_id,
        process_name,
        window_title,
        icon: Some(IconData {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }),
    })
}
