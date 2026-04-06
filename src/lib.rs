//! # gtec
//!
//! Rust library and terminal UI for the **g.tec Unicorn Hybrid Black**
//! 8-channel EEG headset via the [Unicorn C API](https://github.com/unicorn-bi/Unicorn-Hybrid-Black-Windows-APIs),
//! loaded at runtime.
//!
//! ## Cross-platform
//!
//! Works on **Windows** and **Linux** (x86_64). The `Unicorn.dll` /
//! `libunicorn.so` shared library is loaded at runtime via `libloading` —
//! no build-time C dependencies.
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use gtec::prelude::*;
//!
//! let serials = UnicornDevice::scan(true)?;
//! let mut device = UnicornDevice::open(&serials[0])?;
//!
//! println!("Info: {:?}", device.device_info()?);
//!
//! let scans = device.capture(UNICORN_SAMPLING_RATE * 4)?;
//! for s in &scans[..5] {
//!     println!("EEG: {:?}", s.eeg());
//! }
//! ```
//!
//! ## Using as a library
//!
//! ```toml
//! [dependencies]
//! gtec = "0.0.1"
//! gtec = { version = "0.0.1", default-features = false }
//! ```
//!
//! ## Module overview
//!
//! | Module | Purpose |
//! |---|---|
//! | [`ffi`] | Runtime-loaded FFI bindings (16/16 functions) |
//! | [`types`] | `#[repr(C)]` types matching `unicorn.h` |
//! | [`device`] | High-level device API |
//! | [`verify`] | SHA-256 integrity verification |
//! | [`sandbox`] | OS-level network sandboxing |
//! | [`error`] | Error types |
//! | [`prelude`] | Convenience re-exports |

pub mod ffi;
pub mod types;
pub mod error;
pub mod verify;
pub mod sandbox;
pub mod device;

/// Convenience re-exports.
pub mod prelude {
    pub use crate::error::UnicornError;
    pub use crate::types::*;
    pub use crate::device::{UnicornDevice, Scan};
    pub use crate::verify::verify_library;
    pub use crate::sandbox::block_internet;
}
