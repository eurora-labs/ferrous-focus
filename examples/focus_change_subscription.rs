//! Example demonstrating the focus change subscription API
//!
//! This example shows how to use the event-driven focus tracking
//! by subscribing to focus changes via a channel.

use ferrous_focus::subscribe_focus_changes;
use std::time::Duration;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting focus change subscription example...");
    info!("Switch between different applications to see focus changes.");
    info!("Press Ctrl+C to exit.");

    // Subscribe to focus changes
    let receiver = subscribe_focus_changes()?;

    // Set up Ctrl+C handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        info!("\nReceived Ctrl+C, shutting down...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;

    // Listen for focus changes
    let mut event_count = 0;
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(focused_window) => {
                event_count += 1;
                info!(
                    "Focus Event #{}: {} (PID: {:?})",
                    event_count,
                    focused_window.window_title.as_deref().unwrap_or("Unknown"),
                    focused_window.process_id
                );

                if let Some(process_name) = &focused_window.process_name {
                    info!("  Process: {}", process_name);
                }

                if focused_window.icon.is_some() {
                    info!("  Has icon: Yes");
                } else {
                    info!("  Has icon: No");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Continue waiting
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                info!("Focus tracking channel disconnected");
                break;
            }
        }
    }

    info!("Captured {} focus events total", event_count);
    info!("Goodbye!");

    Ok(())
}
