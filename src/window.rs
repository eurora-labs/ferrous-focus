//! Shared types for the cross‑platform focus tracker crate.

use crate::FerrousFocusResult;
use core::fmt;

/// Raw icon data in 32‑bit RGBA format (premultiplied or straight depending on the platform).
#[derive(Clone, PartialEq, Eq)]
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

/// Platform backend contract. Each OS‑specific module must implement this trait.
pub trait FocusProvider {
    /// Retrieve the currently focused window metadata snapshot.
    fn focused_window() -> FerrousFocusResult<FocusedWindow>;
}
