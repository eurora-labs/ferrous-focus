use crate::{FerrousFocusError, FerrousFocusResult, FocusedWindow, RgbaImage};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{
            AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, PropertyNotifyEvent,
        },
    },
    rust_connection::RustConnection,
};

pub fn track_focus<F>(on_focus: F) -> FerrousFocusResult<()>
where
    F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
{
    run(on_focus, None)
}

pub fn track_focus_with_stop<F>(on_focus: F, stop_signal: &AtomicBool) -> FerrousFocusResult<()>
where
    F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
{
    run(on_focus, Some(stop_signal))
}

fn run<F>(mut on_focus: F, stop_signal: Option<&AtomicBool>) -> FerrousFocusResult<()>
where
    F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
{
    // ── X11 setup ──────────────────────────────────────────────────────────────
    let (conn, screen_num) = RustConnection::connect(None).map_err(|e| {
        let error_str = e.to_string();
        // Check if this is a "no display" error
        if error_str.contains("DISPLAY")
            || error_str.contains("display")
            || error_str.contains("No such file or directory")
        {
            FerrousFocusError::NoDisplay
        } else {
            FerrousFocusError::Platform(error_str)
        }
    })?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let net_active_window = atom(&conn, b"_NET_ACTIVE_WINDOW")?;
    let net_wm_name = atom(&conn, b"_NET_WM_NAME")?;
    let net_wm_pid = atom(&conn, b"_NET_WM_PID")?;
    let utf8_string = atom(&conn, b"UTF8_STRING")?;
    let net_wm_icon = atom(&conn, b"_NET_WM_ICON")?;

    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )
    .map_err(|e| FerrousFocusError::Platform(e.to_string()))?;
    conn.flush()
        .map_err(|e| FerrousFocusError::Platform(e.to_string()))?;

    // Track the currently focused window to monitor its title changes
    let mut current_focused_window: Option<u32> = None;

    // ── Event loop ─────────────────────────────────────────────────────────────
    loop {
        // Check stop signal before polling for events
        if let Some(stop) = stop_signal {
            if stop.load(Ordering::Acquire) {
                break;
            }
        }

        let event = match stop_signal {
            Some(_) => {
                // Use polling when stop signal is available
                match conn.poll_for_event() {
                    Ok(Some(e)) => e,
                    Ok(None) => {
                        // No event available, sleep briefly to avoid busy waiting
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                    Err(e) => {
                        info!("X11 error: {e}");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                }
            }
            None => {
                // Use blocking wait when no stop signal
                match conn.wait_for_event() {
                    Ok(e) => e,
                    Err(e) => {
                        info!("X11 error: {e}");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                }
            }
        };

        if let Event::PropertyNotify(PropertyNotifyEvent { atom, window, .. }) = event {
            let mut should_emit_focus_event = false;
            let mut new_window: Option<u32> = None;

            // Check if this is an active window change
            if atom == net_active_window && window == root {
                // Active window changed
                match active_window(&conn, root, net_active_window) {
                    Ok(win) => {
                        new_window = win;
                        should_emit_focus_event = true;

                        // Update monitoring for the new focused window
                        if let Some(old_win) = current_focused_window {
                            // Stop monitoring the old window
                            let _ = conn.change_window_attributes(
                                old_win,
                                &ChangeWindowAttributesAux::new().event_mask(EventMask::NO_EVENT),
                            );
                        }

                        if let Some(new_win) = new_window {
                            // Start monitoring the new window for title changes
                            let _ = conn.change_window_attributes(
                                new_win,
                                &ChangeWindowAttributesAux::new()
                                    .event_mask(EventMask::PROPERTY_CHANGE),
                            );
                            current_focused_window = Some(new_win);
                        } else {
                            current_focused_window = None;
                        }
                    }
                    Err(e) => {
                        info!("Failed to get active window: {}", e);
                        // Continue processing other events instead of crashing
                        continue;
                    }
                }
            }
            // Check if this is a title change on the currently focused window
            else if atom == net_wm_name && Some(window) == current_focused_window {
                // Title changed on the focused window
                new_window = current_focused_window;
                should_emit_focus_event = true;
            }

            if should_emit_focus_event {
                // ── Gather window data ────────────────────────────────────────────
                let win = match new_window {
                    Some(w) => w,
                    None => continue,
                };

                // Handle window property queries with graceful error handling
                let title = window_name(&conn, win, net_wm_name, utf8_string).unwrap_or_else(|e| {
                    info!("Failed to get window title for window {}: {}", win, e);
                    "<unknown title>".to_string()
                });

                let proc = process_name(&conn, win, net_wm_pid).unwrap_or_else(|e| {
                    info!("Failed to get process name for window {}: {}", win, e);
                    "<unknown>".to_string()
                });

                let icon = get_icon_data(&conn, win, net_wm_icon).ok();

                // ── Invoke user-supplied handler ──────────────────────────────────
                if let Err(e) = on_focus(FocusedWindow {
                    process_id: None,
                    process_name: Some(proc),
                    window_title: Some(title),
                    icon,
                }) {
                    info!("Focus event handler failed: {}", e);
                    // Continue processing instead of propagating the error
                }
            }
        }

        conn.flush().map_err(|e| {
            FerrousFocusError::Platform(format!("Failed to flush connection: {}", e))
        })?;
    }

    Ok(())
}

/* ------------------------------------------------------------ */
/* Helper functions                                              */
/* ------------------------------------------------------------ */

fn atom<C: Connection>(conn: &C, name: &[u8]) -> FerrousFocusResult<u32> {
    Ok(conn
        .intern_atom(false, name)
        .map_err(|e| FerrousFocusError::Platform(e.to_string()))?
        .reply()
        .map_err(|e| FerrousFocusError::Platform(e.to_string()))?
        .atom)
}

fn active_window<C: Connection>(
    conn: &C,
    root: u32,
    net_active_window: u32,
) -> FerrousFocusResult<Option<u32>> {
    match conn.get_property(false, root, net_active_window, AtomEnum::WINDOW, 0, 1) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => Ok(reply.value32().and_then(|mut v| v.next())),
            Err(err) => Err(FerrousFocusError::Platform(format!(
                "Failed to get active window: {}",
                err
            ))),
        },
        Err(err) => Err(FerrousFocusError::Platform(format!(
            "Failed to get active window: {}",
            err
        ))),
    }
}

fn window_name<C: Connection>(
    conn: &C,
    window: u32,
    net_wm_name: u32,
    utf8_string: u32,
) -> FerrousFocusResult<String> {
    // Try UTF‑8 first
    match conn.get_property(false, window, net_wm_name, utf8_string, 0, u32::MAX) {
        Ok(cookie) => {
            match cookie.reply() {
                Ok(reply) => {
                    if reply.value_len > 0 {
                        return Ok(String::from_utf8_lossy(&reply.value).into_owned());
                    }

                    // Fallback to the legacy WM_NAME
                    match conn.get_property(
                        false,
                        window,
                        AtomEnum::WM_NAME,
                        AtomEnum::STRING,
                        0,
                        u32::MAX,
                    ) {
                        Ok(cookie) => match cookie.reply() {
                            Ok(reply) => Ok(String::from_utf8_lossy(&reply.value).into_owned()),
                            Err(err) => Err(FerrousFocusError::Platform(format!(
                                "Failed to get window name: {}",
                                err
                            ))),
                        },
                        Err(err) => Err(FerrousFocusError::Platform(format!(
                            "Failed to get window name: {}",
                            err
                        ))),
                    }
                }
                Err(err) => Err(FerrousFocusError::Platform(format!(
                    "Failed to get window name: {}",
                    err
                ))),
            }
        }
        Err(err) => Err(FerrousFocusError::Platform(format!(
            "Failed to get window name: {}",
            err
        ))),
    }
}

fn process_name<C: Connection>(
    conn: &C,
    window: u32,
    net_wm_pid: u32,
) -> FerrousFocusResult<String> {
    // fetch the PID stored in _NET_WM_PID
    let pid = match conn.get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => match reply.value32().and_then(|mut v| v.next()) {
                Some(pid) => pid,
                None => {
                    return Err(FerrousFocusError::Platform(
                        "No PID found for window".to_string(),
                    ));
                }
            },
            Err(err) => {
                return Err(FerrousFocusError::Platform(format!(
                    "Failed to get PID: {}",
                    err
                )));
            }
        },
        Err(err) => {
            return Err(FerrousFocusError::Platform(format!(
                "Failed to get PID: {}",
                err
            )));
        }
    };

    // read /proc/<pid>/comm (single line: executable name)
    match std::fs::read_to_string(format!("/proc/{pid}/comm")).or_else(|_| {
        std::fs::read_link(format!("/proc/{pid}/exe")).map(|p| p.to_string_lossy().into())
    }) {
        Ok(name) => Ok(name.trim_end_matches('\n').to_owned()),
        Err(err) => Err(FerrousFocusError::Platform(format!(
            "Failed to get process name: {}",
            err
        ))),
    }
}

fn get_icon_data<C: Connection>(
    conn: &C,
    window: u32,
    net_wm_icon: u32,
) -> FerrousFocusResult<RgbaImage> {
    match conn.get_property(
        false,
        window,
        net_wm_icon,
        AtomEnum::CARDINAL,
        0,
        u32::MAX / 4, // Limit size to avoid huge icons
    ) {
        Ok(cookie) => {
            match cookie.reply() {
                Ok(reply) => {
                    if reply.value_len == 0 {
                        return Err(FerrousFocusError::Unsupported);
                    }

                    // The _NET_WM_ICON property contains 32-bit values in the format:
                    // [width, height, pixel_data...]
                    // Each pixel is ARGB in native endian format
                    match reply.value32() {
                        Some(values) => {
                            let values: Vec<u32> = values.collect();

                            if values.len() < 2 {
                                return Err(FerrousFocusError::Platform(
                                    "Invalid icon data: missing width/height".to_string(),
                                ));
                            }

                            let width = values[0] as usize;
                            let height = values[1] as usize;

                            if width == 0 || height == 0 {
                                return Err(FerrousFocusError::Platform(
                                    "Invalid icon dimensions".to_string(),
                                ));
                            }

                            let expected_pixels = width.checked_mul(height).ok_or_else(|| {
                                FerrousFocusError::Platform("Icon dimensions overflow".into())
                            })?;
                            let available_pixels = values.len() - 2; // Subtract width and height

                            if available_pixels < expected_pixels {
                                return Err(FerrousFocusError::Platform(format!(
                                    "Insufficient pixel data: expected {}, got {}",
                                    expected_pixels, available_pixels
                                )));
                            }

                            // Convert ARGB u32 values to RGBA u8 bytes
                            let mut pixels = Vec::with_capacity(
                                expected_pixels.checked_mul(4).ok_or_else(|| {
                                    FerrousFocusError::Platform("Icon dimensions overflow".into())
                                })?,
                            );
                            for &argb in &values[2..2 + expected_pixels] {
                                // Extract ARGB components (native endian)
                                let a = ((argb >> 24) & 0xFF) as u8;
                                let r = ((argb >> 16) & 0xFF) as u8;
                                let g = ((argb >> 8) & 0xFF) as u8;
                                let b = (argb & 0xFF) as u8;

                                // Store as RGBA
                                pixels.push(r);
                                pixels.push(g);
                                pixels.push(b);
                                pixels.push(a);
                            }

                            // Create RgbaImage from the pixel data
                            RgbaImage::from_raw(width as u32, height as u32, pixels).ok_or_else(
                                || FerrousFocusError::Platform("Failed to create RgbaImage".into()),
                            )
                        }
                        None => Err(FerrousFocusError::Platform(
                            "Failed to parse icon data as 32-bit values".to_string(),
                        )),
                    }
                }
                Err(err) => Err(FerrousFocusError::Platform(format!(
                    "Failed to get icon property: {}",
                    err
                ))),
            }
        }
        Err(err) => Err(FerrousFocusError::Platform(format!(
            "Failed to request icon property: {}",
            err
        ))),
    }
}
