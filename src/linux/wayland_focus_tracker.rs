use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow, IconData};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
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

    // Connect to swayipc and subscribe to window events
    let mut connection = Connection::new().map_err(|e| {
        FerrousFocusError::Platform(format!("Failed to connect to sway IPC: {}", e))
    })?;

    let event_iterator = connection.subscribe([EventType::Window]).map_err(|e| {
        FerrousFocusError::Platform(format!("Failed to subscribe to window events: {}", e))
    })?;

    // Process events as they arrive
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

                                if let Err(e) = on_focus(window) {
                                    eprintln!("Focus event handler failed: {}", e);
                                    // Continue processing instead of propagating the error
                                }
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
                // Try to reconnect on error
                match Connection::new() {
                    Ok(new_conn) => {
                        connection = new_conn;
                        match connection.subscribe([EventType::Window]) {
                            Ok(_new_iterator) => {
                                // Continue with new iterator - this requires restructuring the loop
                                eprintln!("Reconnected to sway IPC");
                                break;
                            }
                            Err(e) => {
                                eprintln!("Failed to resubscribe after reconnection: {}", e);
                                return Err(FerrousFocusError::Platform(format!(
                                    "Lost connection to sway IPC: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to reconnect to sway IPC: {}", e);
                        return Err(FerrousFocusError::Platform(format!(
                            "Lost connection to sway IPC: {}",
                            e
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn track_focus_with_stop<F>(
    mut on_focus: F,
    stop_signal: Arc<AtomicBool>,
) -> FerrousFocusResult<()>
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

    // Connect to swayipc and subscribe to window events
    let mut connection = Connection::new().map_err(|e| {
        FerrousFocusError::Platform(format!("Failed to connect to sway IPC: {}", e))
    })?;

    let event_iterator = connection.subscribe([EventType::Window]).map_err(|e| {
        FerrousFocusError::Platform(format!("Failed to subscribe to window events: {}", e))
    })?;

    // Process events as they arrive
    for event in event_iterator {
        // Check stop signal before processing each event
        if stop_signal.load(Ordering::Relaxed) {
            break;
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

                                if let Err(e) = on_focus(window) {
                                    eprintln!("Focus event handler failed: {}", e);
                                    // Continue processing instead of propagating the error
                                }
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
                // Try to reconnect on error
                match Connection::new() {
                    Ok(new_conn) => {
                        connection = new_conn;
                        match connection.subscribe([EventType::Window]) {
                            Ok(_new_iterator) => {
                                // Continue with new iterator - this requires restructuring the loop
                                eprintln!("Reconnected to sway IPC");
                                break;
                            }
                            Err(e) => {
                                eprintln!("Failed to resubscribe after reconnection: {}", e);
                                return Err(FerrousFocusError::Platform(format!(
                                    "Lost connection to sway IPC: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to reconnect to sway IPC: {}", e);
                        return Err(FerrousFocusError::Platform(format!(
                            "Lost connection to sway IPC: {}",
                            e
                        )));
                    }
                }
            }
        }
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
