# System Patterns _Optional_

This file documents recurring patterns and standards used in the project.
It is optional, but recommended to be updated as the project evolves.
2025-06-16 13:09:18 - Initial Memory Bank creation for ferrous-focus project.

## Coding Patterns

-   **Error handling**: Centralized error types using `thiserror` crate for consistent error propagation
-   **Platform abstraction**: Common trait definitions with platform-specific implementations
-   **Conditional compilation**: Extensive use of `cfg` attributes for target-specific code paths
-   **Resource management**: RAII patterns for platform-specific resources (window handles, connections)
-   **Logging integration**: Structured logging using `tracing` crate throughout the codebase

## Architectural Patterns

-   **Modular platform support**: Each OS implementation isolated in separate modules
-   **Facade pattern**: Unified API surface hiding platform-specific complexity
-   **Observer pattern**: Event subscription system for focus change notifications
-   **Factory pattern**: Platform-specific tracker creation based on runtime environment
-   **Strategy pattern**: Different focus tracking strategies for X11 vs Wayland on Linux

## Testing Patterns

-   **Integration testing**: Comprehensive test suite covering cross-platform scenarios
-   **Mock/stub patterns**: Test utilities for simulating platform-specific behaviors
-   **Asset-based testing**: Icon validation tests using test asset files
-   **Serial test execution**: Using `serial_test` crate for tests requiring exclusive access
-   **Permission fallback testing**: Graceful degradation when platform permissions are limited
