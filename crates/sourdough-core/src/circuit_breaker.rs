//! Circuit breaker pattern for IPC resilience.
//!
//! Prevents cascading failures when a dependency is unavailable by
//! tracking failure counts and temporarily rejecting calls until the
//! dependency recovers.

/// Simple circuit breaker for IPC resilience.
#[derive(Debug)]
pub struct CircuitBreaker {
    service: String,
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    last_failure: Option<std::time::Instant>,
    reset_timeout: std::time::Duration,
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation.
    Closed,
    /// Too many failures, rejecting calls.
    Open,
    /// Testing if service recovered.
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    #[must_use]
    pub fn new(
        service: impl Into<String>,
        failure_threshold: u32,
        reset_timeout: std::time::Duration,
    ) -> Self {
        Self {
            service: service.into(),
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            last_failure: None,
            reset_timeout,
        }
    }

    /// Check if a call is allowed.
    #[must_use]
    pub fn allow_call(&mut self) -> bool {
        match self.state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() >= self.reset_timeout {
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Record a successful call.
    pub const fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    /// Record a failed call.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Get current state.
    #[must_use]
    pub const fn state(&self) -> CircuitState {
        self.state
    }

    /// Get the service name.
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_allows_and_opens() {
        let mut cb = CircuitBreaker::new("svc", 2, std::time::Duration::from_secs(60));
        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_call());
    }

    #[test]
    fn opens_then_half_open_after_reset() {
        let reset = std::time::Duration::from_millis(20);
        let mut cb = CircuitBreaker::new("svc", 1, reset);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_call());
        std::thread::sleep(reset + std::time::Duration::from_millis(10));
        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
