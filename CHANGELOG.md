# Changelog

## [0.0.1] - 2026-04-06

### Added
- Initial release
- Runtime-loaded FFI bindings to Unicorn C API (16/16 functions)
- Full `#[repr(C)]` type parity with `unicorn.h`
- High-level `UnicornDevice` API (scan, connect, configure, acquire, digital I/O)
- SHA-256 integrity verification of native libraries
- OS-level network sandboxing (seccomp/Seatbelt/Firewall)
- CLI binary + real-time ratatui TUI with 8-channel EEG charts
- Examples: scan, stream, read_eeg
- Cross-platform: Windows (x64), Linux (x64)
- 20 unit tests (no hardware required)
