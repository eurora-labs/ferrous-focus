use std::time::Duration;

/// Configuration for focus tracking behavior
#[derive(Debug, Clone)]
pub struct FocusTrackerConfig {
    /// Polling interval for focus change detection
    /// Default: 100ms
    pub poll_interval: Duration,
}

impl Default for FocusTrackerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(100),
        }
    }
}

impl FocusTrackerConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the polling interval for focus change detection
    ///
    /// # Arguments
    /// * `interval` - The polling interval duration
    ///
    /// # Panics
    /// Panics if the interval is zero or too large (> 10 seconds)
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.validate_poll_interval(interval);
        self.poll_interval = interval;
        self
    }

    /// Set the polling interval in milliseconds
    ///
    /// # Arguments
    /// * `ms` - The polling interval in milliseconds
    ///
    /// # Panics
    /// Panics if the interval is zero or too large (> 10000ms)
    pub fn with_poll_interval_ms(self, ms: u64) -> Self {
        self.with_poll_interval(Duration::from_millis(ms))
    }

    /// Validate the polling interval
    fn validate_poll_interval(&self, interval: Duration) {
        if interval.is_zero() {
            panic!("Poll interval cannot be zero");
        }
        if interval > Duration::from_secs(10) {
            panic!("Poll interval cannot be greater than 10 seconds");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FocusTrackerConfig::default();
        assert_eq!(config.poll_interval, Duration::from_millis(100));
    }

    #[test]
    fn test_builder_pattern() {
        let config = FocusTrackerConfig::new().with_poll_interval_ms(250);
        assert_eq!(config.poll_interval, Duration::from_millis(250));
    }

    #[test]
    fn test_with_poll_interval() {
        let config = FocusTrackerConfig::new().with_poll_interval(Duration::from_millis(500));
        assert_eq!(config.poll_interval, Duration::from_millis(500));
    }

    #[test]
    #[should_panic(expected = "Poll interval cannot be zero")]
    fn test_zero_interval_panics() {
        FocusTrackerConfig::new().with_poll_interval(Duration::from_millis(0));
    }

    #[test]
    #[should_panic(expected = "Poll interval cannot be greater than 10 seconds")]
    fn test_large_interval_panics() {
        FocusTrackerConfig::new().with_poll_interval(Duration::from_secs(11));
    }
}
