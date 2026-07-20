//! Logging setup using tracing-subscriber
//!
//! Initializes the tracing subscriber with environment-driven log level filtering.
//! Log level is controlled by the `RUST_LOG` environment variable.
//! Default level is `info` when `RUST_LOG` is not set.

/// Initialize the tracing subscriber for application logging.
///
/// Call this early in `main()` to capture all log output.
/// Uses `RUST_LOG` env var for filtering, defaults to `info`.
pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("logger initialized");
}
