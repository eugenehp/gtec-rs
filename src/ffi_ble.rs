//! Runtime-loaded FFI bindings to the `libgtecble` BLE library.
//!
//! This is g.tec's official BLE library for the **Unicorn BCI Core-8**
//! (and potentially other future BLE devices). Found in the `gtec-ble`
//! PyPI package from `gtec-medical-engineering/gpype`.
//!
//! **All 9 exported functions** are bound for 100% API parity.
//!
//! | Platform | Library | Source |
//! |---|---|---|
//! | **Windows** (x64) | `libgtecble.dll` | PyPI `gtec-ble` wheel |
//! | **macOS** (universal) | `libgtecble.dylib` | PyPI `gtec-ble` wheel |
//! | **Linux** | ❌ Not available | — |

use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

use crate::error::UnicornError;

// ── Opaque handle ────────────────────────────────────────────────────────────

/// Opaque device handle returned by `GTECBLE_OpenDevice`.
pub type GtecBleHandle = *mut c_void;

// ── Callback types ───────────────────────────────────────────────────────────

/// Callback fired when a device is discovered during scanning.
/// `serials` is a null-terminated array of C strings, `count` is the number.
pub type DeviceDiscoveredCallback =
    unsafe extern "C" fn(serials: *const *const c_char, count: u32, user_data: *mut c_void);

/// Callback fired when data is available from the device.
/// `data` points to `channels * samples` floats.
pub type DataAvailableCallback =
    unsafe extern "C" fn(data: *const f32, channels: u32, samples: u32, user_data: *mut c_void);

// ── Device information struct ────────────────────────────────────────────────

/// Device information returned by `GTECBLE_GetDeviceInformation`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GtecBleDeviceInfo {
    pub model_number: [u8; 64],
    pub serial_number: [u8; 64],
    pub firmware_version: [u8; 64],
    pub hardware_version: [u8; 64],
    pub manufacturer_name: [u8; 64],
    pub channel_count: u32,
    pub sampling_rate: u32,
}

impl GtecBleDeviceInfo {
    fn buf_to_string(buf: &[u8]) -> String {
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..nul]).into_owned()
    }
    pub fn model_str(&self) -> String { Self::buf_to_string(&self.model_number) }
    pub fn serial_str(&self) -> String { Self::buf_to_string(&self.serial_number) }
    pub fn firmware_str(&self) -> String { Self::buf_to_string(&self.firmware_version) }
    pub fn hardware_str(&self) -> String { Self::buf_to_string(&self.hardware_version) }
    pub fn manufacturer_str(&self) -> String { Self::buf_to_string(&self.manufacturer_name) }
}

// ── Function pointer types ───────────────────────────────────────────────────

type FnGetApiVersion = unsafe extern "C" fn() -> f32;
type FnGetLastErrorText = unsafe extern "C" fn() -> *const c_char;
type FnStartScanning = unsafe extern "C" fn() -> i32;
type FnStopScanning = unsafe extern "C" fn() -> i32;
type FnRegisterDeviceDiscovered = unsafe extern "C" fn(
    callback: DeviceDiscoveredCallback,
    user_data: *mut c_void,
) -> i32;
type FnOpenDevice = unsafe extern "C" fn(serial: *const c_char, handle: *mut GtecBleHandle) -> i32;
type FnCloseDevice = unsafe extern "C" fn(handle: *mut GtecBleHandle) -> i32;
type FnRegisterDataAvailable = unsafe extern "C" fn(
    handle: GtecBleHandle,
    callback: DataAvailableCallback,
    user_data: *mut c_void,
) -> i32;
type FnGetDeviceInformation = unsafe extern "C" fn(
    handle: GtecBleHandle,
    info: *mut GtecBleDeviceInfo,
) -> i32;

// ── Library wrapper ──────────────────────────────────────────────────────────

/// Dynamically-loaded `libgtecble` BLE library with all 9 functions.
pub struct GtecBleLib {
    _lib: libloading::Library,

    pub(crate) fn_get_api_version: FnGetApiVersion,
    pub(crate) fn_get_last_error_text: FnGetLastErrorText,
    pub(crate) fn_start_scanning: FnStartScanning,
    pub(crate) fn_stop_scanning: FnStopScanning,
    pub(crate) fn_register_device_discovered: FnRegisterDeviceDiscovered,
    pub(crate) fn_open_device: FnOpenDevice,
    pub(crate) fn_close_device: FnCloseDevice,
    pub(crate) fn_register_data_available: FnRegisterDataAvailable,
    pub(crate) fn_get_device_information: FnGetDeviceInformation,
}

unsafe impl Send for GtecBleLib {}
unsafe impl Sync for GtecBleLib {}

macro_rules! load_fn {
    ($lib:expr, $name:literal, $ty:ty) => {
        *$lib.get::<$ty>($name).map_err(|e| UnicornError::LibraryNotAvailable {
            reason: format!("{}: {}", std::str::from_utf8($name).unwrap_or("?"), e),
        })?
    };
}

impl GtecBleLib {
    fn load() -> Result<Self, UnicornError> {
        let lib_names: &[&str] = if cfg!(target_os = "windows") {
            &["libgtecble", "libgtecble.dll"]
        } else if cfg!(target_os = "macos") {
            &["gtecble", "libgtecble.dylib"]
        } else {
            return Err(UnicornError::LibraryNotAvailable {
                reason: "libgtecble is not available on Linux".into(),
            });
        };

        let mut last_err = String::new();
        for name in lib_names {
            let lib_name = libloading::library_filename(name);
            match unsafe { libloading::Library::new(&lib_name) } {
                Ok(lib) => return unsafe { Self::from_lib(lib) },
                Err(e) => last_err = format!("{:?}: {}", lib_name, e),
            }
        }

        Err(UnicornError::LibraryNotAvailable {
            reason: format!(
                "Could not load libgtecble.\nLast: {}\n\
                 Install via: pip install gtec-ble",
                last_err
            ),
        })
    }

    unsafe fn from_lib(lib: libloading::Library) -> Result<Self, UnicornError> {
        Ok(GtecBleLib {
            fn_get_api_version: load_fn!(lib, b"GTECBLE_GetApiVersion\0", FnGetApiVersion),
            fn_get_last_error_text: load_fn!(lib, b"GTECBLE_GetLastErrorText\0", FnGetLastErrorText),
            fn_start_scanning: load_fn!(lib, b"GTECBLE_StartScanning\0", FnStartScanning),
            fn_stop_scanning: load_fn!(lib, b"GTECBLE_StopScanning\0", FnStopScanning),
            fn_register_device_discovered: load_fn!(lib, b"GTECBLE_RegisterDeviceDiscoveredCallback\0", FnRegisterDeviceDiscovered),
            fn_open_device: load_fn!(lib, b"GTECBLE_OpenDevice\0", FnOpenDevice),
            fn_close_device: load_fn!(lib, b"GTECBLE_CloseDevice\0", FnCloseDevice),
            fn_register_data_available: load_fn!(lib, b"GTECBLE_RegisterDataAvailableCallback\0", FnRegisterDataAvailable),
            fn_get_device_information: load_fn!(lib, b"GTECBLE_GetDeviceInformation\0", FnGetDeviceInformation),
            _lib: lib,
        })
    }
}

// ── Singleton ────────────────────────────────────────────────────────────────

static BLE_LIB: OnceLock<Result<GtecBleLib, String>> = OnceLock::new();

/// Get the global `libgtecble` BLE library handle.
pub fn ble_lib() -> Result<&'static GtecBleLib, UnicornError> {
    BLE_LIB
        .get_or_init(|| GtecBleLib::load().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| UnicornError::LibraryNotAvailable { reason: e.clone() })
}
