# Decision Log

This file records architectural and implementation decisions using a list format.
2025-06-16 13:09:01 - Initial Memory Bank creation for ferrous-focus project.

## Decision

Cross-platform architecture with platform-specific modules

## Rationale

-   Enables optimal implementation for each operating system's native APIs
-   Maintains clean separation of concerns between different platform implementations
-   Allows for platform-specific optimizations while providing unified API surface
-   Facilitates maintenance and testing of individual platform implementations

## Implementation Details

-   Separate modules: src/linux/, src/macos/, src/windows/
-   Common interface through focus_tracker.rs and focused_window.rs
-   Conditional compilation using cfg attributes for target-specific dependencies

---

## Decision

Support for both X11 and Wayland on Linux

## Rationale

-   Linux desktop ecosystem is transitioning from X11 to Wayland
-   Many distributions still use X11 by default or offer both options
-   Different desktop environments have varying levels of Wayland support
-   Comprehensive Linux support requires handling both display servers

## Implementation Details

-   Separate implementations: xorg_focus_tracker.rs and wayland_focus_tracker.rs
-   Runtime detection and fallback mechanisms
-   Platform-specific dependencies: x11rb for X11, wayland-client and swayipc for Wayland

---

## Decision

Event-driven subscription model for focus changes

## Rationale

-   Provides efficient, non-blocking API for applications
-   Allows multiple subscribers to focus change events
-   Reduces polling overhead and improves performance
-   Enables reactive programming patterns in client applications

## Implementation Details

-   Subscription-based API through focus change callbacks
-   Asynchronous event handling where supported by platform
-   Example implementations demonstrating subscription patterns
