use thiserror::Error;

/// The error types for Rusty Advisor.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error: {0}")]
    Msg(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Registration Error: Attempts to create a metric collector already registered with different definition (other description or tag names")]
    MetricAlreadyRegDifferently(),
}

/// A specialized Result type for prometheus.
pub type Result<T> = std::result::Result<T, Error>;
