use crate::{FerrousFocusError, FerrousFocusResult, FocusTrackerConfig, FocusedWindow};
use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::{
    Foundation::{HWND, WPARAM},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
        DeleteObject, GetDIBits, SelectObject,
    },
    UI::WindowsAndMessaging::{
        GCLP_HICON, GCLP_HICONSM, GetClassLongPtrW, ICON_BIG, ICON_SMALL, SendMessageW, WM_GETICON,
    },
};

use super::utils;
use tracing::info;

#[derive(Debug, Clone)]
pub(crate) struct ImplFocusTracker {}

impl ImplFocusTracker {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl ImplFocusTracker {
    pub fn track_focus<F>(&self, on_focus: F, config: &FocusTrackerConfig) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.run(on_focus, None, config)
    }

    pub fn track_focus_with_stop<F>(
        &self,
        on_focus: F,
        stop_signal: &AtomicBool,
        config: &FocusTrackerConfig,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        self.run(on_focus, Some(stop_signal), config)
    }

    fn run<F>(
        &self,
        mut on_focus: F,
        stop_signal: Option<&AtomicBool>,
        config: &FocusTrackerConfig,
    ) -> FerrousFocusResult<()>
    where
        F: FnMut(FocusedWindow) -> FerrousFocusResult<()>,
    {
        // Check if we're in an interactive session
        if !utils::is_interactive_session()? {
            return Err(FerrousFocusError::NotInteractiveSession);
        }

        // Track the previously focused window to avoid duplicate events
        let mut prev_hwnd: Option<HWND> = None;
        let mut prev_title: Option<String> = None;

        // Get initial focused window
        if let Some(hwnd) = utils::get_foreground_window()
            && let Ok((title, process)) = unsafe { utils::get_window_info(hwnd) }
        {
            let icon = get_window_icon(hwnd, &config.icon);
            let process_id = unsafe { utils::get_window_process_id(hwnd) }.unwrap_or_default();
            if let Err(e) = on_focus(FocusedWindow {
                process_id: Some(process_id),
                process_name: Some(process.clone()),
                window_title: Some(title.clone()),
                icon,
            }) {
                info!("Focus event handler failed: {}", e);
            }

            prev_hwnd = Some(hwnd);
            prev_title = Some(title);
        }

        // Main event loop - we'll use polling since Windows event hooks are complex to integrate
        // with Rust's async runtime in a cross-platform way
        loop {
            // Check stop signal before processing
            if let Some(stop) = stop_signal
                && stop.load(Ordering::Relaxed)
            {
                break;
            }

            // Check current foreground window
            if let Some(current_hwnd) = utils::get_foreground_window() {
                let focus_changed = match prev_hwnd {
                    Some(prev) => prev != current_hwnd,
                    None => true,
                };

                match unsafe { utils::get_window_info(current_hwnd) } {
                    Ok((title, process)) => {
                        // Also check if title changed for the same window
                        let title_changed = match &prev_title {
                            Some(prev_t) => prev_t != &title,
                            None => true,
                        };

                        // Trigger handler if either window focus or title has changed
                        if focus_changed || title_changed {
                            let icon = get_window_icon(current_hwnd, &config.icon);
                            let process_id = unsafe { utils::get_window_process_id(current_hwnd) }
                                .unwrap_or_default();
                            if let Err(e) = on_focus(FocusedWindow {
                                process_id: Some(process_id),
                                process_name: Some(process.clone()),
                                window_title: Some(title.clone()),
                                icon,
                            }) {
                                info!("Focus event handler failed: {}", e);
                            }

                            prev_hwnd = Some(current_hwnd);
                            prev_title = Some(title);
                        }
                    }
                    Err(e) => {
                        info!("Failed to get window info: {}", e);
                    }
                }
            } else {
                // No foreground window
                if prev_hwnd.is_some() {
                    prev_hwnd = None;
                    prev_title = None;
                }
            }

            // Sleep to avoid high CPU usage
            std::thread::sleep(config.poll_interval);
        }

        Ok(())
    }
}

/* ------------------------------------------------------------ */
/* Helper functions                                              */
/* ------------------------------------------------------------ */

/// Resize an image to the specified dimensions using Lanczos3 filtering
fn resize_icon(image: image::RgbaImage, target_size: u32) -> image::RgbaImage {
    use image::imageops::FilterType;

    // Only resize if the image is not already the target size
    if image.width() == target_size && image.height() == target_size {
        return image;
    }

    image::imageops::resize(&image, target_size, target_size, FilterType::Lanczos3)
}

/// Get the icon for a window
fn get_window_icon(
    hwnd: HWND,
    icon_config: &crate::config::IconConfig,
) -> Option<image::RgbaImage> {
    unsafe { extract_window_icon(hwnd, icon_config).ok() }
}

/// Extract the icon bitmap from a window handle
///
/// # Safety
/// This function uses unsafe Win32 API calls and assumes the HWND is valid
unsafe fn extract_window_icon(
    hwnd: HWND,
    icon_config: &crate::config::IconConfig,
) -> FerrousFocusResult<image::RgbaImage> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    // Try to get the icon from the window in order of preference:
    // 1. Try WM_GETICON with ICON_BIG
    // 2. Try WM_GETICON with ICON_SMALL
    // 3. Try GetClassLongPtrW with GCLP_HICON
    // 4. Try GetClassLongPtrW with GCLP_HICONSM

    let hicon = unsafe { SendMessageW(hwnd, WM_GETICON, ICON_BIG as WPARAM, 0) };
    let hicon = if hicon != 0 {
        hicon as isize
    } else {
        let hicon = unsafe { SendMessageW(hwnd, WM_GETICON, ICON_SMALL as WPARAM, 0) };
        if hicon != 0 {
            hicon as isize
        } else {
            let hicon = unsafe { GetClassLongPtrW(hwnd, GCLP_HICON) } as isize;
            if hicon != 0 {
                hicon
            } else {
                let hicon = unsafe { GetClassLongPtrW(hwnd, GCLP_HICONSM) } as isize;
                if hicon != 0 {
                    hicon
                } else {
                    return Err(FerrousFocusError::Platform(
                        "No icon found for window".to_string(),
                    ));
                }
            }
        }
    };

    // Get icon information
    let mut icon_info: ICONINFO = unsafe { std::mem::zeroed() };
    if unsafe { GetIconInfo(hicon as _, &mut icon_info) } == 0 {
        return Err(FerrousFocusError::Platform(
            "Failed to get icon info".to_string(),
        ));
    }

    // Extract the color bitmap (hbmColor) or mask bitmap (hbmMask) if color is not available
    let bitmap = if !icon_info.hbmColor.is_null() {
        icon_info.hbmColor
    } else {
        icon_info.hbmMask
    };

    // Get bitmap dimensions
    let hdc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
    if hdc.is_null() {
        unsafe {
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask);
            }
        }
        return Err(FerrousFocusError::Platform(
            "Failed to create DC".to_string(),
        ));
    }

    // Select the bitmap into the DC
    let old_bitmap = unsafe { SelectObject(hdc, bitmap) };

    // Setup BITMAPINFO to get the bitmap data
    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;

    // Get bitmap info
    if unsafe {
        GetDIBits(
            hdc,
            bitmap,
            0,
            0,
            std::ptr::null_mut(),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    } == 0
    {
        unsafe {
            SelectObject(hdc, old_bitmap);
            DeleteDC(hdc);
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask);
            }
        }
        return Err(FerrousFocusError::Platform(
            "Failed to get bitmap info".to_string(),
        ));
    }

    let width = bmi.bmiHeader.biWidth as u32;
    let height = bmi.bmiHeader.biHeight.unsigned_abs();

    if width == 0 || height == 0 {
        unsafe {
            SelectObject(hdc, old_bitmap);
            DeleteDC(hdc);
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask);
            }
        }
        return Err(FerrousFocusError::Platform(
            "Invalid icon dimensions".to_string(),
        ));
    }

    // Setup for 32-bit RGBA
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    bmi.bmiHeader.biHeight = -(height as i32); // Negative for top-down bitmap

    // Allocate buffer for pixel data
    let pixel_count = (width * height) as usize;
    let mut pixels: Vec<u8> = vec![0; pixel_count * 4];

    // Get the actual bitmap bits
    if unsafe {
        GetDIBits(
            hdc,
            bitmap,
            0,
            height,
            pixels.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        )
    } == 0
    {
        unsafe {
            SelectObject(hdc, old_bitmap);
            DeleteDC(hdc);
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask);
            }
        }
        return Err(FerrousFocusError::Platform(
            "Failed to get bitmap bits".to_string(),
        ));
    }

    // Convert BGRA to RGBA
    for i in (0..pixels.len()).step_by(4) {
        pixels.swap(i, i + 2); // Swap B and R
    }

    // Cleanup
    unsafe {
        SelectObject(hdc, old_bitmap);
        DeleteDC(hdc);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask);
        }
        // Note: Don't destroy the icon if it came from GetClassLongPtrW as it's owned by the class
        // Only destroy if it came from WM_GETICON
        let from_wm_geticon = SendMessageW(hwnd, WM_GETICON, ICON_BIG as WPARAM, 0) != 0
            || SendMessageW(hwnd, WM_GETICON, ICON_SMALL as WPARAM, 0) != 0;
        if from_wm_geticon {
            DestroyIcon(hicon as _);
        }
    }

    // Create RgbaImage from pixel data
    let mut image = image::RgbaImage::from_raw(width, height, pixels).ok_or_else(|| {
        FerrousFocusError::Platform("Failed to create RgbaImage from pixel data".to_string())
    })?;

    // Resize the icon if needed
    if let Some(target_size) = icon_config.size {
        image = resize_icon(image, target_size);
    }

    Ok(image)
}
