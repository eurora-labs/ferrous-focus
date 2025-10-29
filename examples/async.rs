//! Async Focus Tracker Example
//!
//! This example demonstrates how to use the async focus tracking capabilities
//! to perform async operations when focus changes occur.
//!
//! To run this example:
//! ```bash
//! cargo run --example async --features async
//! ```

#[cfg(feature = "async")]
use ferrous_focus::{FerrousFocusResult, FocusTracker};
#[cfg(feature = "async")]
use std::time::Duration;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting async focus tracker example...");
    println!("This example demonstrates awaiting async operations in focus callbacks.");
    println!("Switch between different applications to see focus changes.");
    println!("Press Ctrl+C to exit.\n");

    let tracker = FocusTracker::new();

    // Use the async focus tracking method
    tracker
        .track_focus_async(|window| async move {
            println!(
                "🔍 Focus changed to: {}",
                window.window_title.as_deref().unwrap_or("Unknown")
            );

            if let Some(process_name) = &window.process_name {
                println!("   📱 Process: {}", process_name);
            }

            // Check if icon is available
            let icon_status = if window.icon.is_some() {
                "✅ Has icon"
            } else {
                "❌ No icon"
            };
            println!("   Icon: {}", icon_status);

            // Example 1: Simulate async processing with delays
            println!("   ⏳ Performing async processing...");
            simulate_async_processing(&window).await?;

            // Example 2: Async data processing
            println!("   🔢 Processing window data asynchronously...");
            process_window_data(&window).await?;

            println!("   ✨ All async operations complete!\n");

            Ok(())
        })
        .await?;

    println!("👋 Async focus tracking completed!");
    Ok(())
}

/// Simulate async processing that might involve network or computation
#[cfg(feature = "async")]
async fn simulate_async_processing(
    window: &ferrous_focus::FocusedWindow,
) -> FerrousFocusResult<()> {
    // Simulate some async work
    tokio::time::sleep(Duration::from_millis(50)).await;

    println!(
        "   🔄 [ASYNC] Processed focus event for: {}",
        window.process_name.as_deref().unwrap_or("Unknown")
    );

    Ok(())
}

/// Simulate async data processing
#[cfg(feature = "async")]
async fn process_window_data(window: &ferrous_focus::FocusedWindow) -> FerrousFocusResult<()> {
    // Simulate some computation that benefits from async
    tokio::time::sleep(Duration::from_millis(30)).await;

    let title_length = window.window_title.as_ref().map(|t| t.len()).unwrap_or(0);
    let process_length = window.process_name.as_ref().map(|p| p.len()).unwrap_or(0);

    println!(
        "   📊 [DATA] Title length: {}, Process length: {}, Has icon: {}",
        title_length,
        process_length,
        window.icon.is_some()
    );

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature to be enabled.");
    println!("Run with: cargo run --example async --features async");
}
