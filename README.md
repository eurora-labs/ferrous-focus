# ferrous-focus

[![Crates.io](https://img.shields.io/crates/v/ferrous-focus.svg)](https://crates.io/crates/ferrous-focus)
[![Documentation](https://docs.rs/ferrous-focus/badge.svg)](https://docs.rs/ferrous-focus)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

A cross-platform focus tracker for Linux (X11), macOS, and Windows that monitors window focus changes and provides detailed information about the currently focused window.

## Features

-   **Cross-Platform Support**: Works on Linux (X11), macOS, and Windows
-   **Real-time Focus Tracking**: Monitor window focus changes as they happen
-   **Window Information**: Get window title, process name, process ID, and more
-   **Icon Extraction**: Capture and save application icons (with configurable sizes)
-   **Flexible APIs**: Choose between callback-based or channel-based approaches
-   **Configurable Polling**: Adjust polling intervals for your use case
-   **Graceful Shutdown**: Built-in support for controlled stopping
-   **Comprehensive Examples**: Learn from basic and advanced usage patterns

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ferrous-focus = "0.3.1"
```

## Quick Start

Here's the simplest way to get started:

```rust
use ferrous_focus::subscribe_focus_changes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Subscribe to focus changes
    let receiver = subscribe_focus_changes()?;

    // Listen for focus events
    while let Ok(window) = receiver.recv() {
        println!("Focused: {}",
            window.window_title.as_deref().unwrap_or("Unknown"));

        if let Some(process) = &window.process_name {
            println!("Process: {}", process);
        }
    }

    Ok(())
}
```

## Advanced Usage

For more control over the tracking process:

```rust
use ferrous_focus::{FocusTracker, FocusTrackerConfig, IconConfig};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let config = FocusTrackerConfig::new()
        .with_poll_interval_ms(100)
        .with_icon_config(IconConfig::new().with_size(64));

    let tracker = FocusTracker::with_config(config);
    let stop_signal = Arc::new(AtomicBool::new(false));

    // Track focus with full control
    tracker.track_focus_with_stop(|window| {
        println!("Window: {}",
            window.window_title.as_deref().unwrap_or("Unknown"));

        // Save icon if available
        if let Some(icon) = window.icon {
            icon.save("current_app_icon.png")?;
        }

        Ok(())
    }, &stop_signal)?;

    Ok(())
}
```

## Examples

The repository includes comprehensive examples:

-   **`basic.rs`**: Simple focus tracking with minimal setup
-   **`advanced.rs`**: Full-featured example with icon saving and statistics

Run the examples:

```bash
# Basic example
cargo run --example basic

# Advanced example with icon extraction
cargo run --example advanced
```

## Supported Platforms

| Platform | Window System | Status          |
| -------- | ------------- | --------------- |
| Linux    | X11           | ✅ Full support |
| Linux    | Wayland       | ❌ No support   |
| macOS    | Cocoa         | ✅ Full support |
| Windows  | Win32 API     | ✅ Full support |

### Platform Notes

-   **Linux X11**: Full support with window information and icon extraction
-   **Linux Wayland**: No support at the moment, also not clear if it is even possible
-   **macOS**: Requires accessibility permissions for full functionality
-   **Windows**: Works with all supported Windows versions

## Configuration

Customize the focus tracker behavior:

```rust
use ferrous_focus::{FocusTrackerConfig, IconConfig};

let config = FocusTrackerConfig::new()
    .with_poll_interval_ms(50)           // Faster polling (default: 100ms)
    .with_icon_config(
        IconConfig::new()
            .with_size(128)              // Larger icons (default: Whatever size the platform provides)
    );
```

## API Documentation

For detailed API documentation, visit [docs.rs/ferrous-focus](https://docs.rs/ferrous-focus).

## System Requirements

### macOS

-   Accessibility permissions may be required for full functionality
-   Grant permission in System Preferences > Security & Privacy > Accessibility

### Linux

-   X11 development libraries (for X11 support)
-   Works out of the box on most distributions

### Windows

-   No additional requirements
-   Compatible with Windows 7 and later

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
