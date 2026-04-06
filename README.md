# gtec

A Rust library and terminal UI for **g.tec Unicorn** EEG devices via the
[Unicorn C API](https://github.com/unicorn-bi/Unicorn-Hybrid-Black-Windows-APIs)
and [Bluetooth protocol](https://github.com/unicorn-bi/Unicorn-Suite-Hybrid-Black/tree/master/Unicorn%20Bluetooth%20Protocol).

Rust FFI wrapper with 100% API parity — all 16 functions from `unicorn.h` bound,
all types mapped to `#[repr(C)]` Rust equivalents, plus a pure-Rust Bluetooth
protocol implementation for macOS support.

## Installation

```shell
cargo add gtec
```

## Supported devices

| Device | Type | Channels | Resolution | Hz | Connection | Status |
|---|---|---|---|---|---|---|
| **Unicorn Hybrid Black** | EEG headset | 8 EEG + 3 accel + 3 gyro + battery + counter (17 total) | 24-bit | 250 | Classic Bluetooth RFCOMM | ✅ Full support |
| **Unicorn Naked** | OEM board (same HW) | Same as Hybrid Black | 24-bit | 250 | Classic Bluetooth RFCOMM | ✅ Full support (same device) |
| **Unicorn BCI Core-8** | EEG amplifier kit | 8 EEG | 24-bit | 250 | Bluetooth 5 (BLE) | ✅ Full support (`libgtecble` + pure-Rust BLE) |

### Unicorn BCI Core-8 support

The BCI Core-8 is supported via g.tec's official **`libgtecble`** BLE library, found
in the [`gtec-ble`](https://pypi.org/project/gtec-ble/) PyPI package published by
[`gtec-medical-engineering/gpype`](https://github.com/gtec-medical-engineering/gpype)
(g.tec's official Python SDK). The `ffi_ble` module binds all 9 exported functions:

| Function | Purpose |
|---|---|
| `GTECBLE_GetApiVersion` | API version |
| `GTECBLE_GetLastErrorText` | Last error description |
| `GTECBLE_StartScanning` | Start BLE device scan |
| `GTECBLE_StopScanning` | Stop BLE device scan |
| `GTECBLE_RegisterDeviceDiscoveredCallback` | Callback when device found |
| `GTECBLE_OpenDevice` | Connect to device by serial |
| `GTECBLE_CloseDevice` | Disconnect device |
| `GTECBLE_RegisterDataAvailableCallback` | Callback for streaming data |
| `GTECBLE_GetDeviceInformation` | Device info (model, serial, firmware) |

**Platform support for BCI Core-8:**

| Platform | Library | Source |
|---|---|---|
| **Windows** (x64) | `libgtecble.dll` (311 KB) | PyPI `gtec-ble` wheel |
| **macOS** (universal) | `libgtecble.dylib` (927 KB) | PyPI `gtec-ble` wheel |
| **Linux** (x64) | Pure-Rust `ble.rs` via `btleplug` | No native lib needed |

### Other g.tec devices (not supported)

| Device | Why not supported |
|---|---|
| Unicorn tDCS Core-2 | Transcranial stimulator, not EEG |
| g.USBamp | Proprietary `gAPI` (commercial license, not on GitHub) |
| g.Nautilus | Proprietary `gAPI` |
| g.HIamp | Proprietary `gAPI` |

## Cross-platform

Works on **Windows**, **Linux**, and **macOS**.

### Hybrid Black

| Platform | Backend | Source |
|---|---|---|
| **Windows** (x64) | `Unicorn.dll` via `ffi` | [unicorn-bi/Unicorn-Hybrid-Black-Windows-APIs](https://github.com/unicorn-bi/Unicorn-Hybrid-Black-Windows-APIs) → `c-api/Lib/` |
| **Linux** (x64) | `libunicorn.so` via `ffi` | [unicorn-bi/Unicorn-Suite-Hybrid-Black](https://github.com/unicorn-bi/Unicorn-Suite-Hybrid-Black) → `Unicorn Linux C API/x64/Lib/` |
| **macOS** | Pure-Rust `protocol.rs` over BT RFCOMM | No native lib needed |

### BCI Core-8

| Platform | Backend | Source |
|---|---|---|
| **Windows** (x64) | `libgtecble.dll` via `ffi_ble` | PyPI [`gtec-ble`](https://pypi.org/project/gtec-ble/) v2.0.1 |
| **macOS** (universal) | `libgtecble.dylib` via `ffi_ble` | PyPI [`gtec-ble`](https://pypi.org/project/gtec-ble/) v2.0.1 |
| **Linux** (x64) | Pure-Rust `ble.rs` via `btleplug` | No native lib needed |
| **All OS** | Pure-Rust `ble.rs` via `btleplug` | `cargo add gtec --features ble` |

The `ble` feature (enabled by default) provides a pure-Rust BLE backend using
[`btleplug`](https://crates.io/crates/btleplug) that works on **all platforms**
without any native library. GATT UUIDs were extracted from g.tec's official
`libgtecble.dylib`.

**Note:** g.tec does not provide a macOS native library, but this crate includes
a pure-Rust Bluetooth protocol implementation (`protocol.rs`) that enables macOS
support by communicating directly with the device over RFCOMM. The protocol was
implemented from the official [UnicornBluetoothProtocol.pdf](https://github.com/unicorn-bi/Unicorn-Suite-Hybrid-Black/tree/master/Unicorn%20Bluetooth%20Protocol).

### Protocol details

The Unicorn uses classic Bluetooth RFCOMM with a simple binary protocol:

| Element | Format |
|---|---|
| Start acquisition | `0x61 0x7C 0x87` → ACK `0x00 0x00 0x00` |
| Stop acquisition | `0x63 0x5C 0xC5` → ACK `0x00 0x00 0x00` |
| Payload (250 Hz) | 45 bytes per sample |

Payload structure:
```text
[0..1]   Header:        0xC0 0x00
[2]      Battery:       4-bit voltage in low nibble
[3..26]  EEG 1–8:       3 bytes each (24-bit signed, big-endian)
[27..32] Accelerometer:  2 bytes each (16-bit signed, little-endian) × 3 axes
[33..38] Gyroscope:      2 bytes each (16-bit signed, little-endian) × 3 axes
[39..42] Counter:        4 bytes (32-bit unsigned, little-endian)
[43..44] Footer:         0x0D 0x0A
```

Scale factors:
- **EEG**: `raw × 4500000 / 50331642` → µV
- **Accelerometer**: `raw / 4096` → g
- **Gyroscope**: `raw / 32.8` → °/s
- **Battery**: `(raw & 0x0F) / 15 × 100` → %

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
    ├── ffi.rs            # Runtime-loaded Unicorn C API for Hybrid Black (16 functions)
    ├── ffi_ble.rs        # Runtime-loaded libgtecble BLE API for BCI Core-8 (9 functions)
    ├── ble.rs            # Pure-Rust BLE backend for BCI Core-8 via btleplug (all OS)
    ├── types.rs          # #[repr(C)] types matching unicorn.h
    ├── protocol.rs       # Pure-Rust BT protocol (45-byte payload decode)
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
    └── types_tests.rs    # FFI type layout + constant + protocol tests
```

## Dependencies

| Crate | Purpose |
|---|---|
| [libloading](https://crates.io/crates/libloading) | Runtime DLL/so loading |
| [btleplug](https://crates.io/crates/btleplug) | Cross-platform BLE (optional, `ble` feature) |
| [tokio](https://crates.io/crates/tokio) | Async runtime for BLE (optional, `ble` feature) |
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

25 unit tests covering FFI type layouts, enum values, ABI struct sizes,
protocol payload decoding (verified against official g.tec example), SHA-256
correctness, and constant validation — all without hardware.

## License

[MIT](./LICENSE)
