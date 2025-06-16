# Product Context

This file provides a high-level overview of the project and the expected product that will be created. Initially it is based upon projectBrief.md (if provided) and all other available project-related information in the working directory. This file is intended to be updated as the project evolves, and should be used to inform all other modes of the project's goals and context.
2025-06-16 13:08:20 - Initial Memory Bank creation for ferrous-focus project.

## Project Overview

**ferrous-focus** is a cross-platform focus tracker library written in Rust that monitors window focus changes across Linux (X11/Wayland), macOS, and Windows operating systems.

## Project Goal

-   Provide a unified, cross-platform API for tracking window focus changes
-   Support major desktop environments and window managers across three primary operating systems
-   Deliver a reliable, performant library that developers can integrate into their applications

## Key Features

-   **Cross-platform support**: Linux (X11 and Wayland), macOS, and Windows
-   **Focus change detection**: Real-time monitoring of window focus events
-   **Window information extraction**: Access to focused window metadata
-   **Icon tracking capabilities**: Support for extracting and tracking window icons
-   **Multiple Linux backends**: Support for both X11 and Wayland display servers
-   **Subscription-based API**: Event-driven architecture for focus change notifications

## Overall Architecture

-   **Platform-specific implementations**: Separate modules for each OS (linux/, macos/, windows/)
-   **Unified API surface**: Common interface through focus_tracker.rs and focused_window.rs
-   **Error handling**: Centralized error types and handling via error.rs
-   **Testing infrastructure**: Comprehensive test suite including integration tests
-   **Example applications**: Demonstration code showing various usage patterns
-   **Cross-compilation support**: Conditional compilation for different target platforms

## Technical Stack

-   **Language**: Rust (Edition 2024)
-   **Linux dependencies**: x11rb, wayland-client, swayipc
-   **macOS dependencies**: dispatch2, objc2 family crates
-   **Windows dependencies**: windows-sys with Win32 API bindings
-   **Development tools**: tracing for logging, thiserror for error handling
