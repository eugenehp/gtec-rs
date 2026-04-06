//! FFI types matching `unicorn.h` from the Unicorn C API.
//!
//! All types are `#[repr(C)]` for ABI compatibility with the
//! `Unicorn.dll` / `libunicorn.so` native library.

use std::fmt;

// ── Constants ────────────────────────────────────────────────────────────────

/// The sampling rate of the Unicorn Brain Interface (Hz).
pub const UNICORN_SAMPLING_RATE: u32 = 250;

/// The number of EEG channels.
pub const UNICORN_EEG_CHANNELS_COUNT: usize = 8;

/// The number of accelerometer channels.
pub const UNICORN_ACCELEROMETER_CHANNELS_COUNT: usize = 3;

/// The number of gyroscope channels.
pub const UNICORN_GYROSCOPE_CHANNELS_COUNT: usize = 3;

/// The total number of available channels (EEG + accel + gyro + battery + counter + validation).
pub const UNICORN_TOTAL_CHANNELS_COUNT: usize = 17;

/// Index of the first EEG channel in the configuration.
pub const UNICORN_EEG_CONFIG_INDEX: usize = 0;

/// Index of the first accelerometer channel in the configuration.
pub const UNICORN_ACCELEROMETER_CONFIG_INDEX: usize = 8;

/// Index of the first gyroscope channel in the configuration.
pub const UNICORN_GYROSCOPE_CONFIG_INDEX: usize = 11;

/// Index of the battery level channel in the configuration.
pub const UNICORN_BATTERY_CONFIG_INDEX: usize = 14;

/// Index of the counter channel in the configuration.
pub const UNICORN_COUNTER_CONFIG_INDEX: usize = 15;

/// Index of the validation indicator channel in the configuration.
pub const UNICORN_VALIDATION_CONFIG_INDEX: usize = 16;

/// Maximum length of the serial number.
pub const UNICORN_SERIAL_LENGTH_MAX: usize = 14;

/// Maximum length of the device version string.
pub const UNICORN_DEVICE_VERSION_LENGTH_MAX: usize = 6;

/// Maximum length of the firmware version string.
pub const UNICORN_FIRMWARE_VERSION_LENGTH_MAX: usize = 12;

/// Maximum string length.
pub const UNICORN_STRING_LENGTH_MAX: usize = 255;

/// The number of digital output channels.
pub const UNICORN_NUMBER_OF_DIGITAL_OUTPUTS: usize = 8;

/// The supported device version prefix.
pub const UNICORN_SUPPORTED_DEVICE_VERSION: &str = "1.";

// ── Error Codes ──────────────────────────────────────────────────────────────

/// The operation completed successfully.
pub const UNICORN_ERROR_SUCCESS: i32 = 0;
/// One of the specified parameters does not contain a valid value.
pub const UNICORN_ERROR_INVALID_PARAMETER: i32 = 1;
/// The initialization of the Bluetooth adapter failed.
pub const UNICORN_ERROR_BLUETOOTH_INIT_FAILED: i32 = 2;
/// The operation could not be performed because the Bluetooth socket failed.
pub const UNICORN_ERROR_BLUETOOTH_SOCKET_FAILED: i32 = 3;
/// The device could not be opened.
pub const UNICORN_ERROR_OPEN_DEVICE_FAILED: i32 = 4;
/// The configuration is invalid.
pub const UNICORN_ERROR_INVALID_CONFIGURATION: i32 = 5;
/// The acquisition buffer is full.
pub const UNICORN_ERROR_BUFFER_OVERFLOW: i32 = 6;
/// The acquisition buffer is empty.
pub const UNICORN_ERROR_BUFFER_UNDERFLOW: i32 = 7;
/// The operation is not allowed during acquisition or non-acquisition.
pub const UNICORN_ERROR_OPERATION_NOT_ALLOWED: i32 = 8;
/// The operation could not complete because of connection problems.
pub const UNICORN_ERROR_CONNECTION_PROBLEM: i32 = 9;
/// The device is not supported with this API.
pub const UNICORN_ERROR_UNSUPPORTED_DEVICE: i32 = 10;
/// The specified connection handle is invalid.
pub const UNICORN_ERROR_INVALID_HANDLE: i32 = 0xFFFFFFFEu32 as i32;
/// An unspecified error occurred.
pub const UNICORN_ERROR_GENERAL_ERROR: i32 = 0xFFFFFFFFu32 as i32;

/// Convert an error code to a human-readable name.
pub fn error_name(code: i32) -> &'static str {
    match code {
        UNICORN_ERROR_SUCCESS => "Success",
        UNICORN_ERROR_INVALID_PARAMETER => "Invalid parameter",
        UNICORN_ERROR_BLUETOOTH_INIT_FAILED => "Bluetooth init failed",
        UNICORN_ERROR_BLUETOOTH_SOCKET_FAILED => "Bluetooth socket failed",
        UNICORN_ERROR_OPEN_DEVICE_FAILED => "Open device failed",
        UNICORN_ERROR_INVALID_CONFIGURATION => "Invalid configuration",
        UNICORN_ERROR_BUFFER_OVERFLOW => "Buffer overflow",
        UNICORN_ERROR_BUFFER_UNDERFLOW => "Buffer underflow",
        UNICORN_ERROR_OPERATION_NOT_ALLOWED => "Operation not allowed",
        UNICORN_ERROR_CONNECTION_PROBLEM => "Connection problem",
        UNICORN_ERROR_UNSUPPORTED_DEVICE => "Unsupported device",
        UNICORN_ERROR_INVALID_HANDLE => "Invalid handle",
        UNICORN_ERROR_GENERAL_ERROR => "General error",
        _ => "Unknown error",
    }
}

// ── Type aliases ─────────────────────────────────────────────────────────────

/// The handle type for an open Unicorn device session.
pub type UnicornHandle = u64;

/// Device serial number (fixed-size C string).
pub type UnicornDeviceSerial = [u8; UNICORN_SERIAL_LENGTH_MAX];

/// Device version string (fixed-size C string).
pub type UnicornDeviceVersion = [u8; UNICORN_DEVICE_VERSION_LENGTH_MAX];

/// Firmware version string (fixed-size C string).
pub type UnicornFirmwareVersion = [u8; UNICORN_FIRMWARE_VERSION_LENGTH_MAX];

/// Boolean type matching the C API's `BOOL` (i32).
pub type UnicornBool = i32;

// ── Structures ───────────────────────────────────────────────────────────────

/// Information about a single amplifier channel.
#[repr(C)]
#[derive(Clone)]
pub struct UnicornAmplifierChannel {
    /// Channel name (null-terminated C string).
    pub name: [u8; 32],
    /// Channel unit (null-terminated C string).
    pub unit: [u8; 32],
    /// Channel input range: `[min, max]`.
    pub range: [f32; 2],
    /// Channel enabled flag. Non-zero = enabled.
    pub enabled: UnicornBool,
}

impl UnicornAmplifierChannel {
    /// Channel name as a Rust string.
    pub fn name_str(&self) -> String {
        let nul = self.name.iter().position(|&b| b == 0).unwrap_or(32);
        String::from_utf8_lossy(&self.name[..nul]).into_owned()
    }

    /// Channel unit as a Rust string.
    pub fn unit_str(&self) -> String {
        let nul = self.unit.iter().position(|&b| b == 0).unwrap_or(32);
        String::from_utf8_lossy(&self.unit[..nul]).into_owned()
    }

    /// Whether this channel is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled != 0
    }
}

impl fmt::Debug for UnicornAmplifierChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AmplifierChannel")
            .field("name", &self.name_str())
            .field("unit", &self.unit_str())
            .field("range", &self.range)
            .field("enabled", &self.is_enabled())
            .finish()
    }
}

/// Amplifier configuration holding all channel settings.
#[repr(C)]
#[derive(Clone)]
pub struct UnicornAmplifierConfiguration {
    /// Array of channel configurations.
    pub channels: [UnicornAmplifierChannel; UNICORN_TOTAL_CHANNELS_COUNT],
}

impl fmt::Debug for UnicornAmplifierConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AmplifierConfiguration")
            .field("channels", &self.channels.iter().map(|c| c.name_str()).collect::<Vec<_>>())
            .finish()
    }
}

/// Additional information about the Unicorn device.
#[repr(C)]
#[derive(Clone)]
pub struct UnicornDeviceInformation {
    /// Number of EEG channels.
    pub number_of_eeg_channels: u16,
    /// Serial number.
    pub serial: UnicornDeviceSerial,
    /// Firmware version.
    pub firmware_version: UnicornFirmwareVersion,
    /// Device version.
    pub device_version: UnicornDeviceVersion,
    /// PCB version number.
    pub pcb_version: [u8; 4],
    /// Enclosure version number.
    pub enclosure_version: [u8; 4],
}

impl UnicornDeviceInformation {
    fn buf_to_string(buf: &[u8]) -> String {
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..nul]).into_owned()
    }

    /// Serial number as a Rust string.
    pub fn serial_str(&self) -> String {
        Self::buf_to_string(&self.serial)
    }

    /// Firmware version as a Rust string.
    pub fn firmware_version_str(&self) -> String {
        Self::buf_to_string(&self.firmware_version)
    }

    /// Device version as a Rust string.
    pub fn device_version_str(&self) -> String {
        Self::buf_to_string(&self.device_version)
    }
}

impl fmt::Debug for UnicornDeviceInformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceInformation")
            .field("eeg_channels", &self.number_of_eeg_channels)
            .field("serial", &self.serial_str())
            .field("firmware", &self.firmware_version_str())
            .field("device_version", &self.device_version_str())
            .field("pcb_version", &self.pcb_version)
            .field("enclosure_version", &self.enclosure_version)
            .finish()
    }
}

/// Information about the Bluetooth adapter (Windows only).
#[repr(C)]
#[derive(Clone)]
pub struct UnicornBluetoothAdapterInfo {
    /// Bluetooth adapter name.
    pub name: [u8; UNICORN_STRING_LENGTH_MAX],
    /// Bluetooth adapter manufacturer.
    pub manufacturer: [u8; UNICORN_STRING_LENGTH_MAX],
    /// Whether this is the recommended (delivered) adapter.
    pub is_recommended_device: UnicornBool,
    /// Whether the adapter reports a problem.
    pub has_problem: UnicornBool,
}

impl UnicornBluetoothAdapterInfo {
    /// Adapter name as a Rust string.
    pub fn name_str(&self) -> String {
        let nul = self.name.iter().position(|&b| b == 0).unwrap_or(UNICORN_STRING_LENGTH_MAX);
        String::from_utf8_lossy(&self.name[..nul]).into_owned()
    }

    /// Adapter manufacturer as a Rust string.
    pub fn manufacturer_str(&self) -> String {
        let nul = self.manufacturer.iter().position(|&b| b == 0).unwrap_or(UNICORN_STRING_LENGTH_MAX);
        String::from_utf8_lossy(&self.manufacturer[..nul]).into_owned()
    }
}

impl fmt::Debug for UnicornBluetoothAdapterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BluetoothAdapterInfo")
            .field("name", &self.name_str())
            .field("manufacturer", &self.manufacturer_str())
            .field("is_recommended", &(self.is_recommended_device != 0))
            .field("has_problem", &(self.has_problem != 0))
            .finish()
    }
}

/// EEG channel names in the standard 10-20 system order.
pub const EEG_CHANNEL_NAMES: [&str; UNICORN_EEG_CHANNELS_COUNT] = [
    "EEG 1", "EEG 2", "EEG 3", "EEG 4",
    "EEG 5", "EEG 6", "EEG 7", "EEG 8",
];

/// All default channel names in order.
pub const ALL_CHANNEL_NAMES: [&str; UNICORN_TOTAL_CHANNELS_COUNT] = [
    "EEG 1", "EEG 2", "EEG 3", "EEG 4",
    "EEG 5", "EEG 6", "EEG 7", "EEG 8",
    "Accelerometer X", "Accelerometer Y", "Accelerometer Z",
    "Gyroscope X", "Gyroscope Y", "Gyroscope Z",
    "Battery Level",
    "Counter",
    "Validation Indicator",
];
