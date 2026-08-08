//! Platform-aware shutdown signal handling.
//!
//! Every primal needs to handle graceful shutdown. The mechanism differs:
//! - **Unix/macOS/iOS**: SIGTERM, SIGINT (via `tokio::signal::unix`)
//! - **Windows**: Ctrl+C, Ctrl+Break (via `tokio::signal::ctrl_c`)
//! - **Android**: Same as Unix (Linux kernel)
//! - **WASM**: Not applicable (no signals)
//!
//! This module provides a single `shutdown_signal()` future that resolves when
//! the process should begin graceful shutdown, regardless of platform.
//!
//! # Usage
//!
//! ```no_run
//! use sourdough_core::platform_signal::shutdown_signal;
//!
//! # #[tokio::main]
//! # async fn main() {
//! tokio::select! {
//!     _ = shutdown_signal() => {
//!         println!("Shutting down gracefully...");
//!     }
//!     _ = run_server() => {}
//! }
//! # }
//! # async fn run_server() {}
//! ```

/// Wait for a platform-appropriate shutdown signal.
///
/// Returns when the process should begin graceful termination.
///
/// - **Unix** (Linux, macOS, iOS, Android, BSD): SIGTERM or SIGINT
/// - **Windows**: Ctrl+C
///
/// This function can be called multiple times — each call gets its own
/// signal listener. Use `tokio::select!` to race it against your main loop.
pub async fn shutdown_signal() {
    let signal = platform_signal_impl().await;
    tracing::info!(signal = %signal, "shutdown signal received");
}

/// Wait for shutdown and return the signal name (for logging/diagnostics).
pub async fn shutdown_signal_named() -> &'static str {
    let signal = platform_signal_impl().await;
    tracing::info!(signal = %signal, "shutdown signal received");
    signal
}

#[cfg(unix)]
async fn platform_signal_impl() -> &'static str {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigterm =
        signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    let mut sigint =
        signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => "SIGTERM",
        _ = sigint.recv() => "SIGINT",
    }
}

#[cfg(not(unix))]
async fn platform_signal_impl() -> &'static str {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to register Ctrl+C handler");
    "ctrl_c"
}

/// Register a shutdown hook that will run cleanup logic on signal.
///
/// The provided future is spawned as a background task that waits for the
/// shutdown signal and then executes the cleanup closure.
///
/// Returns a `tokio::task::JoinHandle` that resolves after cleanup completes.
pub fn on_shutdown<F, Fut>(cleanup: F) -> tokio::task::JoinHandle<()>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        shutdown_signal().await;
        cleanup().await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shutdown_signal_is_send_and_static() {
        fn assert_send<T: Send>(_: &T) {}
        fn assert_static<T: 'static>(_: &T) {}

        let fut = shutdown_signal();
        assert_send(&fut);
        assert_static(&fut);
        // Don't actually await — it would block forever without a signal
        drop(fut);
    }

    #[tokio::test]
    async fn on_shutdown_returns_join_handle() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let handle = on_shutdown(move || async move {
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Handle exists and is valid (we can abort it)
        handle.abort();

        // Flag was NOT set because we aborted before signal
        assert!(!flag.load(Ordering::SeqCst));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn signal_handler_registers_without_panic() {
        use tokio::signal::unix::{SignalKind, signal};

        // Verify we can register signal handlers (the core mechanism)
        let sigterm = signal(SignalKind::terminate());
        assert!(sigterm.is_ok());

        let sigint = signal(SignalKind::interrupt());
        assert!(sigint.is_ok());
    }
}
