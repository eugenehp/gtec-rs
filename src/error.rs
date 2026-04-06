//! Error types for the g.tec Unicorn API wrapper.

/// All errors that can occur within the g.tec Unicorn API.
#[derive(Debug, thiserror::Error)]
pub enum UnicornError {
    /// The Unicorn shared library could not be loaded.
    #[error("Unicorn library not available: {reason}")]
    LibraryNotAvailable { reason: String },

    /// An SDK operation returned an error code.
    #[error("Unicorn error (code {code}): {message}")]
    SdkError { code: i32, message: String },

    /// No devices were found during scanning.
    #[error("No Unicorn device found")]
    NoDeviceFound,

    /// The device is not connected.
    #[error("Device not connected")]
    NotConnected,

    /// A feature is not supported on this platform.
    #[error("Not supported: {0}")]
    NotSupported(String),

    /// A null pointer was returned where a valid pointer was expected.
    #[error("Null pointer returned from SDK")]
    NullPointer,
}
