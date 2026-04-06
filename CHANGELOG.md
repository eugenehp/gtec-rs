# Changelog

## [0.0.2] - 2026-04-06

### Added
- Pure-Rust BLE backend (`ble.rs`) for BCI Core-8 via `btleplug` — enables Linux support
- GATT UUIDs extracted from official `libgtecble.dylib` (service `39a76676-...`, characteristics `B5211405`–`B521140A`)
- `ffi_ble` module for `libgtecble` native library (9/9 functions)
- BCI Core-8 now fully supported on all three OS

### Changed
- Default features now include `ble` (btleplug + tokio)
- Updated README with full platform × device × backend matrix

## [0.0.1] - 2026-04-06

### Added
- Initial release
- Runtime-loaded FFI bindings to Unicorn C API (16/16 functions)
- Pure-Rust Bluetooth protocol implementation from official spec
- Protocol decode verified against official example payload
- SHA-256 integrity verification, network sandboxing
- CLI + TUI, examples, 25 tests
