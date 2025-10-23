//! Shared types for the cross‑platform focus tracker crate.
use fxhash::FxHasher;
use std::hash::{Hash, Hasher};

// Re-export the RgbaImage from the image crate for convenience
pub use image::RgbaImage;

/// Extension trait to add utility methods to RgbaImage for this crate
pub trait IconExt {
    /// Hash the icon data using FxHasher
    fn hash_icon(&self) -> u64;
}

impl IconExt for RgbaImage {
    /// Hash the icon data using FxHasher
    fn hash_icon(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.width().hash(&mut hasher);
        self.height().hash(&mut hasher);
        self.as_raw().hash(&mut hasher);
        hasher.finish()
    }
}

/// Snapshot of the currently focused window.
#[derive(Debug, Clone)]
pub struct FocusedWindow {
    /// Process ID of the focused window.
    pub process_id: Option<u32>,
    /// Reported process name (e.g. "firefox", "chrome", "code").
    pub process_name: Option<String>,
    /// Full window title/caption as provided by the OS.
    pub window_title: Option<String>,
    /// Icon as RGBA image (may be `None` if not retrievable on the platform).
    pub icon: Option<RgbaImage>,
}

impl PartialEq for FocusedWindow {
    fn eq(&self, other: &Self) -> bool {
        self.process_id == other.process_id
            && self.process_name == other.process_name
            && self.window_title == other.window_title
            && match (&self.icon, &other.icon) {
                (Some(a), Some(b)) => a.dimensions() == b.dimensions() && a.as_raw() == b.as_raw(),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Eq for FocusedWindow {}
