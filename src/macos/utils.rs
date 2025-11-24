use crate::{FocusedWindow, config::IconConfig, error::FerrousFocusResult};
use objc2::ClassType;
use objc2::msg_send;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::AnyObject;
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSCompositingOperation, NSImage, NSRunningApplication,
    NSWorkspace,
};
use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString, ns_string};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ProcessSerialNumber {
    high: u32,
    low: u32,
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut AnyObject;
    fn AXUIElementCopyAttributeValue(
        element: *const AnyObject,
        attribute: *const AnyObject,
        value: *mut *mut AnyObject,
    ) -> i32;
    fn CFRelease(cf: *const AnyObject);
    #[allow(deprecated)]
    fn GetFrontProcess(psn: *mut ProcessSerialNumber) -> i32;
    #[allow(deprecated)]
    fn GetProcessPID(psn: *const ProcessSerialNumber, pid: *mut i32) -> i32;
}

const NO_ERR: i32 = 0;
const K_AX_ERROR_SUCCESS: i32 = 0;
const K_AX_ERROR_APIDISABLED: i32 = -25211;

pub fn get_frontmost_window_info(icon_config: &IconConfig) -> FerrousFocusResult<FocusedWindow> {
    autoreleasepool(|_pool| {
        let mut psn = ProcessSerialNumber { high: 0, low: 0 };
        let err = unsafe { GetFrontProcess(&mut psn) };

        if err != NO_ERR {
            return Err(crate::error::FerrousFocusError::Platform(
                "Failed to get front process".to_string(),
            ));
        }

        let mut pid: i32 = 0;
        let err = unsafe { GetProcessPID(&psn, &mut pid) };

        if err != NO_ERR {
            return Err(crate::error::FerrousFocusError::Platform(
                "Failed to get PID from process".to_string(),
            ));
        }

        let running_app =
            unsafe { NSRunningApplication::runningApplicationWithProcessIdentifier(pid) };

        let process_name = if let Some(ref app) = running_app {
            let name = unsafe { app.localizedName() };
            name.map(|n| n.to_string())
        } else {
            None
        };

        let window_title = get_window_title_via_accessibility(pid)?;

        let icon = if let Some(app) = running_app {
            get_app_icon(&app, icon_config)?
        } else {
            None
        };

        Ok(FocusedWindow {
            process_id: Some(pid as u32),
            window_title,
            process_name,
            icon,
        })
    })
}

fn get_window_title_via_accessibility(pid: i32) -> FerrousFocusResult<Option<String>> {
    let app_element = unsafe { AXUIElementCreateApplication(pid) };
    if app_element.is_null() {
        return Ok(None);
    }

    let focused_window_key = ns_string!("AXFocusedWindow");
    let mut focused_window: *mut AnyObject = std::ptr::null_mut();
    let result = unsafe {
        AXUIElementCopyAttributeValue(
            app_element,
            focused_window_key as *const NSString as *const AnyObject,
            &mut focused_window,
        )
    };

    unsafe { CFRelease(app_element) };

    if result == K_AX_ERROR_APIDISABLED {
        return Err(crate::error::FerrousFocusError::PermissionDenied);
    }

    if result != K_AX_ERROR_SUCCESS || focused_window.is_null() {
        return Ok(None);
    }

    let title_key = ns_string!("AXTitle");
    let mut title: *mut AnyObject = std::ptr::null_mut();
    let result = unsafe {
        AXUIElementCopyAttributeValue(
            focused_window,
            title_key as *const NSString as *const AnyObject,
            &mut title,
        )
    };

    unsafe { CFRelease(focused_window) };

    if result != K_AX_ERROR_SUCCESS || title.is_null() {
        return Ok(None);
    }

    let title_str = unsafe {
        let retained = Retained::from_raw(title as *mut NSString).unwrap();
        retained.to_string()
    };

    Ok(Some(title_str))
}

fn get_app_icon(
    app: &NSRunningApplication,
    icon_config: &IconConfig,
) -> FerrousFocusResult<Option<image::RgbaImage>> {
    let bundle_url = unsafe { app.bundleURL() };
    if bundle_url.is_none() {
        return Ok(None);
    }
    let bundle_url = bundle_url.unwrap();

    let workspace = unsafe { NSWorkspace::sharedWorkspace() };
    let icon = unsafe { workspace.iconForFile(&bundle_url.path().unwrap()) };

    let rgba_image = nsimage_to_rgba(&icon, icon_config)?;
    Ok(Some(rgba_image))
}

fn nsimage_to_rgba(
    image: &NSImage,
    icon_config: &IconConfig,
) -> FerrousFocusResult<image::RgbaImage> {
    let icon_size = icon_config.get_size_or_default() as f64;

    let size = NSSize {
        width: icon_size,
        height: icon_size,
    };

    unsafe { image.setSize(size) };

    let rect = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size,
    };

    let bitmap_rep = unsafe {
        NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            msg_send![NSBitmapImageRep::class(), alloc],
            std::ptr::null_mut(),
            icon_size as isize,
            icon_size as isize,
            8,
            4,
            true,
            false,
            ns_string!("NSCalibratedRGBColorSpace"),
            0,
            0,
        )
    };

    if bitmap_rep.is_none() {
        return Err(crate::error::FerrousFocusError::Platform(
            "Failed to create bitmap representation".to_string(),
        ));
    }
    let bitmap_rep = bitmap_rep.unwrap();

    let ns_graphics_context_class = objc2::class!(NSGraphicsContext);
    let graphics_context: *mut AnyObject = unsafe {
        msg_send![
            ns_graphics_context_class,
            graphicsContextWithBitmapImageRep: &*bitmap_rep
        ]
    };

    unsafe {
        let _: () = msg_send![ns_graphics_context_class, saveGraphicsState];
        let _: () = msg_send![ns_graphics_context_class, setCurrentContext: graphics_context];
    }

    let from_rect = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize {
            width: 0.0,
            height: 0.0,
        },
    };
    unsafe {
        image.drawInRect_fromRect_operation_fraction(
            rect,
            from_rect,
            NSCompositingOperation::Copy,
            1.0,
        );
    }

    unsafe {
        let _: () = msg_send![ns_graphics_context_class, restoreGraphicsState];
    }

    let empty_dict = NSDictionary::new();
    let png_data = unsafe {
        bitmap_rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &empty_dict)
    };

    if png_data.is_none() {
        return Err(crate::error::FerrousFocusError::Platform(
            "Failed to get PNG data from bitmap".to_string(),
        ));
    }
    let png_data = png_data.unwrap();

    let bytes = unsafe {
        let data_ptr: *const std::ffi::c_void = msg_send![&*png_data, bytes];
        std::slice::from_raw_parts(data_ptr as *const u8, png_data.len())
    };

    let rgba_image = image::load_from_memory(bytes)
        .map_err(|e| {
            crate::error::FerrousFocusError::Platform(format!(
                "Failed to load image from PNG data: {}",
                e
            ))
        })?
        .to_rgba8();

    Ok(rgba_image)
}
