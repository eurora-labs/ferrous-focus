//! Shared types for the cross‑platform focus tracker crate.
use core::fmt;
use fxhash::FxHasher;
use std::hash::{Hash, Hasher};

/// Raw icon data in 32‑bit RGBA format (premultiplied or straight depending on the platform).
#[derive(Clone, PartialEq, Eq, Default)]
pub struct IconData {
    /// Pixel width of the icon.
    pub width: usize,
    /// Pixel height of the icon.
    pub height: usize,
    /// Pixel buffer, row‑major, 4 bytes per pixel (R, G, B, A).
    pub pixels: Vec<u8>,
}

impl fmt::Debug for IconData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IconData")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pixels_len", &self.pixels.len())
            .finish()
    }
}

impl IconData {
    /// Convert icon data to bytes slice for hashing
    pub fn as_bytes(&self) -> &[u8] {
        &self.pixels
    }

    /// Hash the icon data using FxHasher
    pub fn hash_icon(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.width.hash(&mut hasher);
        self.height.hash(&mut hasher);
        self.pixels.hash(&mut hasher);
        hasher.finish()
    }
}

/// Snapshot of the currently focused window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedWindow {
    /// Process ID of the focused window.
    pub process_id: Option<u32>,
    /// Reported process name (e.g. "firefox", "chrome", "code").
    pub process_name: Option<String>,
    /// Full window title/caption as provided by the OS.
    pub window_title: Option<String>,
    /// Raw icon data (may be `None` if not retrievable on the platform).
    pub icon: Option<IconData>,
}
