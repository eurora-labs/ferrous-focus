//! Phase 4 - Icon Data Verification Tests
//!
//! These tests verify that icon data is properly formatted and can be
//! differentiated between different applications.

mod util;

use ferrous_focus::{FocusTracker, FocusedWindow, IconData};
use fxhash::FxHasher;
use serial_test::serial;
use std::hash::{Hash, Hasher};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tracing::info;
use util::*;

/// Helper function to hash icon bytes using FxHasher
fn hash_icon_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = FxHasher::default();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Test that PNG format icons have correct PNG header and can be decoded
#[test]
#[serial]
fn test_icon_format_png() {
    if !should_run_integration_tests() {
        info!("Skipping integration test - INTEGRATION_TEST=1 not set");
        return;
    }

    if let Err(e) = setup_test_environment() {
        info!("Skipping test due to environment setup failure: {}", e);
        return;
    }

    // This test is primarily for Windows/macOS where icons are returned as PNG bytes
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        let focus_events = Arc::new(Mutex::new(Vec::<FocusedWindow>::new()));
        let focus_events_clone = focus_events.clone();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = stop_signal.clone();

        // Spawn a test window
        match spawn_test_window("PNG Icon Test Window") {
            Ok(mut child) => {
                info!("Spawned test window for PNG icon test");

                // Focus the window
                if let Err(e) = focus_window(&mut child) {
                    info!("Warning: Failed to focus window: {}", e);
                }

                // Start focus tracking
                let tracker_handle = std::thread::spawn(move || {
                    let tracker = FocusTracker::new();
                    let _ = tracker.track_focus_with_stop(
                        move |window: FocusedWindow| -> ferrous_focus::FerrousFocusResult<()> {
                            if let Ok(mut events) = focus_events_clone.lock() {
                                events.push(window);
                            }
                            Ok(())
                        },
                        &stop_signal_clone,
                    );
                });

                // Let it run briefly
                std::thread::sleep(Duration::from_millis(1000));
                stop_signal.store(true, Ordering::Relaxed);
                let _ = tracker_handle.join();

                // Check for PNG format icons
                if let Ok(events) = focus_events.lock() {
                    for event in events.iter() {
                        if let Some(icon) = &event.icon {
                            let bytes = icon.as_bytes();

                            // On Windows/macOS, we expect PNG format
                            // Check PNG header: [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
                            if bytes.len() >= 8 {
                                let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
                                if bytes[..8] == png_header {
                                    info!("Found valid PNG header in icon data");

                                    // Try to decode with image crate
                                    match image::load_from_memory(bytes) {
                                        Ok(img) => {
                                            info!(
                                                "Successfully decoded PNG icon: {}x{}",
                                                img.width(),
                                                img.height()
                                            );
                                            assert!(img.width() > 0 && img.height() > 0);
                                            // Verify dimensions match IconData
                                            assert_eq!(
                                                img.width() as usize,
                                                icon.width,
                                                "PNG width should match IconData width"
                                            );
                                            assert_eq!(
                                                img.height() as usize,
                                                icon.height,
                                                "PNG height should match IconData height"
                                            );
                                        }
                                        Err(e) => {
                                            info!("Failed to decode PNG icon: {}", e);
                                        }
                                    }
                                } else {
                                    info!("Icon data does not have PNG header (may be raw RGBA)");
                                }
                            }
                        }
                    }
                }

                // Cleanup
                if let Err(e) = cleanup_child_process(child) {
                    info!("Warning: Failed to cleanup child process: {}", e);
                }
            }
            Err(e) => {
                info!("Could not spawn test window: {}", e);
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        info!("PNG format test is primarily for Windows/macOS platforms");
    }
}

/// Test that RGBA format icons have correct dimensions
#[test]
#[serial]
fn test_icon_format_rgba() {
    if !should_run_integration_tests() {
        info!("Skipping integration test - INTEGRATION_TEST=1 not set");
        return;
    }

    if let Err(e) = setup_test_environment() {
        info!("Skipping test due to environment setup failure: {}", e);
        return;
    }

    // This test is primarily for X11 systems using _NET_WM_ICON
    #[cfg(target_os = "linux")]
    {
        let focus_events = Arc::new(Mutex::new(Vec::<FocusedWindow>::new()));
        let focus_events_clone = focus_events.clone();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = stop_signal.clone();

        // Spawn a test window
        match spawn_test_window("RGBA Icon Test Window") {
            Ok(mut child) => {
                info!("Spawned test window for RGBA icon test");

                // Focus the window
                if let Err(e) = focus_window(&mut child) {
                    info!("Warning: Failed to focus window: {}", e);
                }

                // Start focus tracking
                let tracker_handle = std::thread::spawn(move || {
                    let tracker = FocusTracker::new();
                    let _ = tracker.track_focus_with_stop(
                        move |window: FocusedWindow| -> ferrous_focus::FerrousFocusResult<()> {
                            if let Ok(mut events) = focus_events_clone.lock() {
                                events.push(window);
                            }
                            Ok(())
                        },
                        &stop_signal_clone,
                    );
                });

                // Let it run briefly
                std::thread::sleep(Duration::from_millis(1000));
                stop_signal.store(true, Ordering::Relaxed);
                let _ = tracker_handle.join();

                // Check for RGBA format icons
                if let Ok(events) = focus_events.lock() {
                    for event in events.iter() {
                        if let Some(icon) = &event.icon {
                            // For X11 _NET_WM_ICON, we expect RGBA format
                            // Assert width * height * 4 == data.len()
                            let expected_size = icon.width * icon.height * 4;
                            let actual_size = icon.pixels.len();

                            info!(
                                "Icon dimensions: {}x{}, expected size: {}, actual size: {}",
                                icon.width, icon.height, expected_size, actual_size
                            );

                            // Always assert for RGBA format on Linux X11
                            assert_eq!(
                                expected_size, actual_size,
                                "Icon data size should match width * height * 4 for RGBA format. Expected: {} bytes, Actual: {} bytes",
                                expected_size, actual_size
                            );
                            info!("RGBA icon format validation passed");
                        }
                    }
                }

                // Cleanup
                if let Err(e) = cleanup_child_process(child) {
                    info!("Warning: Failed to cleanup child process: {}", e);
                }
            }
            Err(e) => {
                info!("Could not spawn test window: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        info!("RGBA format test is primarily for Linux X11 systems");
    }
}

/// Test that different applications have different icon hashes
#[test]
#[serial]
fn test_icon_diff_between_apps() {
    if !should_run_integration_tests() {
        info!("Skipping integration test - INTEGRATION_TEST=1 not set");
        return;
    }

    if let Err(e) = setup_test_environment() {
        info!("Skipping test due to environment setup failure: {}", e);
        return;
    }

    let mut icon_hashes = Vec::new();

    // Test with two different icon files from tests/assets
    let test_configs = vec![
        ("Window with Icon 1", "tests/assets/icon_1.png"),
        ("Window with Icon 2", "tests/assets/icon_2.png"),
    ];

    for (window_title, icon_path) in test_configs {
        let focus_events = Arc::new(Mutex::new(Vec::<FocusedWindow>::new()));
        let focus_events_clone = focus_events.clone();
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = stop_signal.clone();

        match spawn_test_window_with_icon(window_title, icon_path) {
            Ok(mut child) => {
                info!(
                    "✓ Successfully spawned test window: {} with icon: {}",
                    window_title, icon_path
                );

                // Focus the window
                if let Err(e) = focus_window(&mut child) {
                    info!("Warning: Failed to focus window: {}", e);
                }

                // Start focus tracking
                let tracker_handle = std::thread::spawn(move || {
                    let tracker = FocusTracker::new();
                    let _ = tracker.track_focus_with_stop(
                        move |window: FocusedWindow| -> ferrous_focus::FerrousFocusResult<()> {
                            if let Ok(mut events) = focus_events_clone.lock() {
                                events.push(window);
                            }
                            Ok(())
                        },
                        &stop_signal_clone,
                    );
                });

                // Let it run briefly
                std::thread::sleep(Duration::from_millis(1000));
                stop_signal.store(true, Ordering::Relaxed);
                let _ = tracker_handle.join();

                // Capture icon hash
                if let Ok(events) = focus_events.lock() {
                    info!(
                        "Captured {} focus events for '{}'",
                        events.len(),
                        window_title
                    );
                    for event in events.iter() {
                        info!(
                            "Event - Title: {:?}, Process: {:?}, Icon present: {}",
                            event.window_title,
                            event.process_name,
                            event.icon.is_some()
                        );
                        if let Some(icon) = &event.icon {
                            let hash = icon.hash_icon();
                            info!("✓ Icon hash for '{}': {}", window_title, hash);
                            icon_hashes.push((
                                window_title.to_string(),
                                hash,
                                icon_path.to_string(),
                            ));
                            break; // Only need one hash per window
                        }
                    }
                    if events.is_empty() {
                        info!("No focus events captured for '{}'", window_title);
                    }
                } else {
                    info!("Failed to lock focus events for '{}'", window_title);
                }

                // Cleanup
                if let Err(e) = cleanup_child_process(child) {
                    info!("Warning: Failed to cleanup child process: {}", e);
                }

                // Small delay between windows
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => {
                info!(
                    "✗ Could not spawn test window '{}' with icon '{}': {}",
                    window_title, icon_path, e
                );
            }
        }
    }

    // Verify that we have different hashes for different windows
    info!("Collected {} icon hashes", icon_hashes.len());

    // Assert that we collected at least 2 icon hashes
    assert!(
        icon_hashes.len() >= 2,
        "Expected at least 2 icon hashes, but only collected {}. This indicates that icon data was not captured properly.",
        icon_hashes.len()
    );

    // Check that hashes are different - this is the critical assertion
    for i in 0..icon_hashes.len() {
        for j in (i + 1)..icon_hashes.len() {
            let (title1, hash1, icon_path1) = &icon_hashes[i];
            let (title2, hash2, icon_path2) = &icon_hashes[j];

            assert_ne!(
                hash1, hash2,
                "Icon hashes must be different for different icon files. Found identical hashes for '{}' (icon: {}) and '{}' (icon: {}): {}. This indicates that the icons are not being differentiated properly.",
                title1, icon_path1, title2, icon_path2, hash1
            );

            info!(
                "✓ Icon hashes correctly differ between '{}' and '{}': {} vs {}",
                title1, title2, hash1, hash2
            );
        }
    }

    info!("✓ All icon hashes are distinct - test passed!");
}

/// Test the hash_icon helper function directly
#[test]
fn test_hash_icon_helper() {
    // Create test icon data
    let icon1 = IconData {
        width: 32,
        height: 32,
        pixels: vec![255u8; 32 * 32 * 4], // All white pixels
    };

    let icon2 = IconData {
        width: 32,
        height: 32,
        pixels: vec![0u8; 32 * 32 * 4], // All black pixels
    };

    let icon3 = IconData {
        width: 16, // Different dimensions
        height: 16,
        pixels: vec![255u8; 16 * 16 * 4],
    };

    // Test that different icons have different hashes
    let hash1 = icon1.hash_icon();
    let hash2 = icon2.hash_icon();
    let hash3 = icon3.hash_icon();

    info!("Hash 1 (32x32 white): {}", hash1);
    info!("Hash 2 (32x32 black): {}", hash2);
    info!("Hash 3 (16x16 white): {}", hash3);

    assert_ne!(
        hash1, hash2,
        "Different pixel data should produce different hashes"
    );
    assert_ne!(
        hash1, hash3,
        "Different dimensions should produce different hashes"
    );
    assert_ne!(
        hash2, hash3,
        "Different dimensions and pixels should produce different hashes"
    );

    // Test that same icon produces same hash
    let hash1_again = icon1.hash_icon();
    assert_eq!(hash1, hash1_again, "Same icon should produce same hash");
}

/// Test the as_bytes helper function
#[test]
fn test_as_bytes_helper() {
    let test_pixels = vec![255u8, 0u8, 128u8, 64u8];
    let icon = IconData {
        width: 1,
        height: 1,
        pixels: test_pixels.clone(),
    };

    let bytes = icon.as_bytes();
    assert_eq!(
        bytes, &test_pixels,
        "as_bytes should return reference to pixel data"
    );

    // Test that we can hash the bytes
    let hash = hash_icon_bytes(bytes);
    info!("Hash of test bytes: {}", hash);
    assert!(hash != 0, "Hash should not be zero for non-empty data");
}
