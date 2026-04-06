# gtec

A Rust library and terminal UI for the **g.tec Unicorn Hybrid Black** 8-channel
EEG headset via the [Unicorn C API](https://github.com/unicorn-bi/Unicorn-Hybrid-Black-Windows-APIs).

Rust FFI wrapper with 100% API parity — all 16 functions from `unicorn.h` bound,
all types mapped to `#[repr(C)]` Rust equivalents, SHA-256 verification, and
OS-level network sandboxing.

## Installation

```shell
cargo add gtec
```

## Supported hardware

| Device | Channels | Sampling Rate | Connection |
|---|---|---|---|
| Unicorn Hybrid Black | 8 EEG + 3 accel + 3 gyro + battery + counter + validation (17 total) | 250 Hz | Bluetooth |

## Cross-platform

Works on **Windows** and **Linux** (x86_64). The Unicorn library is loaded at
runtime via `libloading` — no build-time C dependencies.

**Note:** No macOS native library is provided by g.tec.

### Native library sources

| Platform | Repository | File |
|---|---|---|
| **Windows** | [Unicorn-Hybrid-Black-Windows-APIs](https://github.com/unicorn-bi/Unicorn-Hybrid-Black-Windows-APIs) | `Unicorn.dll` |
| **Linux** | [Unicorn-Suite-Hybrid-Black](https://github.com/unicorn-bi/Unicorn-Suite-Hybrid-Black) | `libunicorn.so` |

## Quick start

```rust
use gtec::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let serials = UnicornDevice::scan(true)?;
    let mut device = UnicornDevice::open(&serials[0])?;

    println!("Info: {:?}", device.device_info()?);

    let scans = device.capture(UNICORN_SAMPLING_RATE * 4)?;
    for s in &scans[..5] {
        println!("EEG: {:?}", s.eeg());
    }

    device.close()?;
    Ok(())
}
```

## Project layout

```
gtec-rs/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── LICENSE
└── src/
    ├── lib.rs            # Crate root + prelude
    ├── main.rs           # CLI binary
    ├── bin/tui.rs        # ratatui TUI (8-channel EEG charts)
    ├── ffi.rs            # Runtime-loaded Unicorn C API (16 functions)
    ├── types.rs          # #[repr(C)] types matching unicorn.h
    ├── device.rs         # High-level device API
    ├── error.rs          # Error types
    ├── verify.rs         # SHA-256 integrity verification
    └── sandbox.rs        # OS-level network sandboxing
├── sdk/
│   ├── download.sh       # Download + verify native libraries
│   └── checksums.sha256  # Pinned SHA-256 hashes
├── examples/
│   ├── scan.rs           # Device discovery
│   ├── stream.rs         # Signal streaming
│   └── read_eeg.rs       # 4-second EEG capture
└── tests/
    └── types_tests.rs    # FFI type layout + constant tests
```

## Dependencies

| Crate | Purpose |
|---|---|
| [libloading](https://crates.io/crates/libloading) | Runtime DLL/so loading |
| [thiserror](https://crates.io/crates/thiserror) | Error type derivation |
| [log](https://crates.io/crates/log) | Logging facade |
| [env_logger](https://crates.io/crates/env_logger) | Log output |
| [libc](https://crates.io/crates/libc) | seccomp syscalls (Linux) |
| [ratatui](https://ratatui.rs) | Terminal UI (optional) |
| [crossterm](https://github.com/crossterm-rs/crossterm) | Terminal backend (optional) |

## Running tests

```bash
cargo test
```

## License

[MIT](./LICENSE)
