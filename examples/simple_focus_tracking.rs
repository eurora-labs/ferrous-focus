//! Simple example demonstrating the improved track_focus_with_stop API
//!
//! This example shows how the new API allows callers to manage AtomicBool ownership
//! without requiring Arc wrapping, making it easier to use and test.

use ferrous_focus::{FerrousFocusResult, FocusTracker, FocusedWindow};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::info;
use tracing_subscriber::fmt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt::init();
    info!("Starting simple focus tracking example...");

    // Create a stop signal - no Arc wrapper needed!
    // We use Arc here only for sharing between threads, but the API itself doesn't require it
    use std::sync::Arc;
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = stop_signal.clone();

    // Create the focus tracker
    let tracker = FocusTracker::new();

    // Start tracking in a separate thread
    let tracker_handle = std::thread::spawn(move || {
        let result = tracker.track_focus_with_stop(
            |window: FocusedWindow| -> FerrousFocusResult<()> {
                info!("Focus changed to: {:?}", window.window_title);
                if let Some(process) = &window.process_name {
                    info!("  Process: {}", process);
                }
                Ok(())
            },
            &stop_signal_clone, // Pass reference directly - much cleaner than before!
        );

        match result {
            Ok(_) => info!("Focus tracking completed successfully"),
            Err(e) => info!("Focus tracking failed: {}", e),
        }
    });

    // Let it run for 5 seconds
    info!("Tracking focus for 5 seconds...");
    std::thread::sleep(Duration::from_secs(5));

    // Signal to stop
    info!("Stopping focus tracking...");
    stop_signal.store(true, Ordering::Relaxed);

    // Wait for completion
    if let Err(e) = tracker_handle.join() {
        info!("Failed to join tracker thread: {:?}", e);
    }

    info!("Example completed!");
    Ok(())
}
