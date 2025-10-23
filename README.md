# ferrous-focus

[![Crates.io](https://img.shields.io/crates/v/ferrous-focus.svg)](https://crates.io/crates/ferrous-focus)
[![Documentation](https://docs.rs/ferrous-focus/badge.svg)](https://docs.rs/ferrous-focus)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

A cross-platform focus tracker for Linux (X11/Wayland), macOS, and Windows that monitors window focus changes and provides detailed information about the currently focused window.

## Features

-   **Cross-platform support**: Works on Linux (X11 and Wayland), macOS, and Windows
-   **Real-time focus tracking**: Monitor window focus changes as they happen
-   **Window metadata**: Access window title, process name, process ID, and icon data
-   **Icon extraction**: Retrieve window icons in RGBA format
-   **Event-driven API**: Subscribe to focus changes via channels or callbacks
-   **Graceful shutdown**: Stop tracking with atomic boolean signals
-   **Comprehensive platform support**:
    -   Linux: X11 and Wayland display servers
    -   macOS: Native Cocoa/AppKit integration
    -   Windows: Win32 API integration

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ferrous-focus = "0.2.6"
```

## Quick Start

### Basic Focus Tracking

Track focus changes with a simple callback:

```rust
use ferrous_focus::{FocusTracker, FocusedWindow, FerrousFocusResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a focus tracker
    let tracker = FocusTracker::new();

    // Create a stop signal
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_clone = stop_signal.clone();

    // Start tracking in a separate thread
    let handle = std::thread::spawn(move || {
        tracker.track_focus_with_stop(
            |window: FocusedWindow| -> FerrousFocusResult<()> {
                println!("Focus changed to: {:?}", window.window_title);
                if let Some(process) = &window.process_name {
                    println!("  Process: {}", process);
                }
                Ok(())
            },
            &stop_clone,
        )
    });

    // Let it run for 5 seconds
    std::thread::sleep(Duration::from_secs(5));

    // Stop tracking
    stop_signal.store(true, Ordering::Relaxed);
    handle.join().unwrap()?;

    Ok(())
}
```

### Event Subscription

Subscribe to focus changes via a channel:

```rust
use ferrous_focus::subscribe_focus_changes;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Subscribe to focus changes
    let receiver = subscribe_focus_changes()?;

    // Listen for focus events
    loop {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(focused_window) => {
                println!(
                    "Focus Event: {} (PID: {:?})",
                    focused_window.window_title.as_deref().unwrap_or("Unknown"),
                    focused_window.process_id
                );

                if let Some(process_name) = &focused_window.process_name {
                    println!("  Process: {}", process_name);
                }

                if focused_window.icon.is_some() {
                    println!("  Has icon: Yes");
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Continue waiting
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                println!("Focus tracking channel disconnected");
                break;
            }
        }
    }

    Ok(())
}
```

### Icon Extraction

Extract and save window icons:

```rust
use ferrous_focus::{FocusTracker, FocusedWindow, IconData, FerrousFocusResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn save_icon_to_file(icon_data: &IconData, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    if icon_data.pixels.is_empty() || icon_data.width == 0 || icon_data.height == 0 {
        return Err("Invalid icon data".into());
    }

    // Create an image buffer from the icon data (requires 'image' crate)
    let img = image::RgbaImage::from_raw(
        icon_data.width as u32,
        icon_data.height as u32,
        icon_data.pixels.clone(),
    )
    .ok_or("Failed to create image from icon data")?;

    img.save(filename)?;
    println!("Saved icon to: {}", filename);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = FocusTracker::new();
    let stop_signal = Arc::new(AtomicBool::new(false));
    let mut icon_counter = 0;

    tracker.track_focus_with_stop(
        move |window: FocusedWindow| -> FerrousFocusResult<()> {
            if let Some(icon) = window.icon {
                if icon.width > 0 && icon.height > 0 && !icon.pixels.is_empty() {
                    icon_counter += 1;
                    let filename = format!("icon_{:03}.png", icon_counter);

                    if let Err(e) = save_icon_to_file(&icon, &filename) {
                        println!("Failed to save icon: {}", e);
                    }
                }
            }
            Ok(())
        },
        &stop_signal,
    )?;

    Ok(())
}
```

## API Reference

### Core Types

#### `FocusTracker`

The main interface for tracking window focus changes.

```rust
impl FocusTracker {
    pub fn new() -> Self
    pub fn track_focus<F>(&self, on_focus: F) -> FerrousFocusResult<()>
    pub fn track_focus_with_stop<F>(&self, on_focus: F, stop_signal: &AtomicBool) -> FerrousFocusResult<()>
    pub fn subscribe_focus_changes(&self) -> FerrousFocusResult<mpsc::Receiver<FocusedWindow>>
}
```

#### `FocusedWindow`

Information about the currently focused window.

```rust
pub struct FocusedWindow {
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
    pub window_title: Option<String>,
    pub icon: Option<IconData>,
}
```

#### `IconData`

Raw icon data in RGBA format.

```rust
pub struct IconData {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // RGBA format, 4 bytes per pixel
}
```

### Convenience Functions

```rust
// Subscribe to focus changes (creates a new FocusTracker internally)
pub fn subscribe_focus_changes() -> FerrousFocusResult<std::sync::mpsc::Receiver<FocusedWindow>>
```

## Platform Support

### Linux

-   **X11**: Full support via `x11rb`
-   **Wayland**: Support via `wayland-client` and `swayipc`
-   Automatic detection and fallback between display servers

### macOS

-   Native integration using Objective-C bindings
-   Support for Cocoa/AppKit window management

### Windows

-   Win32 API integration
-   Support for all Windows desktop applications

## Examples

The repository includes several examples demonstrating different usage patterns:

-   [`simple_focus_tracking.rs`](examples/simple_focus_tracking.rs) - Basic focus tracking with stop signal
-   [`focus_change_subscription.rs`](examples/focus_change_subscription.rs) - Event-driven focus tracking via channels
-   [`focused_icon_tracking.rs`](examples/focused_icon_tracking.rs) - Extract and save window icons
-   [`spawn_window.rs`](examples/spawn_window.rs) - Helper for creating test windows

Run examples with:

```bash
cargo run --example simple_focus_tracking
cargo run --example focus_change_subscription
cargo run --example focused_icon_tracking
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running Examples

```bash
# Basic focus tracking
cargo run --example simple_focus_tracking

# Focus change subscription
cargo run --example focus_change_subscription

# Icon tracking and extraction
cargo run --example focused_icon_tracking

# Create a test window
cargo run --example spawn_window -- --title "My Test Window"
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
