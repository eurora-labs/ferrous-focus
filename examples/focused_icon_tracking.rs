//! Simple example that tracks focus and saves the focused app's icon to a file
//!
//! This example demonstrates how to:
//! 1. Track focus changes using FocusTracker
//! 2. Extract IconData from the focused window
//! 3. Save the icon as a PNG file for inspection
//!
//! Usage: cargo run --example focused_icon_display_simple

use base64::prelude::*;
use ferrous_focus::{FerrousFocusResult, FocusTracker, FocusedWindow};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

fn save_icon_to_file(
    icon_data: &Vec<u8>,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let b64 = BASE64_STANDARD.decode(icon_data)?;
    let img = image::load_from_memory(&b64)?.to_rgba8();

    // Save as PNG
    img.save(filename)?;
    info!("Saved icon to: {}", filename);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting Focused App Icon Display (Simple) example...");
    info!("This will track focus changes and save icons to PNG files.");
    info!("Switch between different applications to see their icons saved!");
    info!("Press Ctrl+C to exit gracefully.");

    // Create a stop signal
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = stop_signal.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        info!("Received interrupt signal (Ctrl+C), shutting down gracefully...");
        stop_signal_clone.store(true, Ordering::SeqCst);
    })?;

    // Create the focus tracker
    let tracker = FocusTracker::new();

    let mut icon_counter = 0;

    // Start tracking focus
    let result = tracker.track_focus_with_stop(
        move |window: FocusedWindow| -> FerrousFocusResult<()> {
            info!("Focus changed to: {:?}", window.window_title);
            if let Some(process) = &window.process_name {
                info!("  Process: {}", process);
            }

            // Handle the icon
            if let Some(icon) = window.icon {
                icon_counter += 1;
                let filename = format!("examples/recorded_icons/icon_{icon_counter:03}.png");

                match save_icon_to_file(&icon, &filename) {
                    Ok(_) => {
                        info!("  ✓ Icon saved successfully as {}", filename);
                    }
                    Err(e) => {
                        info!("  ✗ Failed to save icon: {}", e);
                    }
                }
            } else {
                info!("  ✗ No icon available");
            }

            Ok(())
        },
        &stop_signal,
    );

    match result {
        Ok(_) => info!("Focus tracking completed successfully"),
        Err(e) => info!("Focus tracking failed: {}", e),
    }

    info!("Example completed! Check the current directory for saved icon files.");
    Ok(())
}
