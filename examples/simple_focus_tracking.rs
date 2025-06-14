//! Simple example demonstrating the improved track_focus_with_stop API
//!
//! This example shows how the new API allows callers to manage AtomicBool ownership
//! without requiring Arc wrapping, making it easier to use and test.

use ferrous_focus::{FerrousFocusResult, FocusTracker, FocusedWindow};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting simple focus tracking example...");

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
                println!("Focus changed to: {:?}", window.window_title);
                if let Some(process) = &window.process_name {
                    println!("  Process: {}", process);
                }
                Ok(())
            },
            &stop_signal_clone, // Pass reference directly - much cleaner than before!
        );

        match result {
            Ok(_) => println!("Focus tracking completed successfully"),
            Err(e) => eprintln!("Focus tracking failed: {}", e),
        }
    });

    // Let it run for 5 seconds
    println!("Tracking focus for 5 seconds...");
    std::thread::sleep(Duration::from_secs(5));

    // Signal to stop
    println!("Stopping focus tracking...");
    stop_signal.store(true, Ordering::Relaxed);

    // Wait for completion
    if let Err(e) = tracker_handle.join() {
        eprintln!("Failed to join tracker thread: {:?}", e);
    }

    println!("Example completed!");
    Ok(())
}
