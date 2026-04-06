//! Runtime-loaded FFI bindings to the Unicorn C API.
//!
//! Loads `Unicorn.dll` / `libunicorn.so` at runtime via `libloading`.
//! **All 16 exported functions** from `unicorn.h` are bound for 100% API parity.

use std::ffi::c_char;
use std::sync::OnceLock;

use crate::error::UnicornError;
use crate::types::*;

// ── Function pointer types ───────────────────────────────────────────────────

type FnGetApiVersion = unsafe extern "C" fn() -> f32;
type FnGetLastErrorText = unsafe extern "C" fn() -> *const c_char;
type FnGetBluetoothAdapterInfo = unsafe extern "C" fn(*mut UnicornBluetoothAdapterInfo) -> i32;
type FnGetAvailableDevices = unsafe extern "C" fn(*mut UnicornDeviceSerial, *mut u32, UnicornBool) -> i32;
type FnOpenDevice = unsafe extern "C" fn(*const c_char, *mut UnicornHandle) -> i32;
type FnCloseDevice = unsafe extern "C" fn(*mut UnicornHandle) -> i32;
type FnStartAcquisition = unsafe extern "C" fn(UnicornHandle, UnicornBool) -> i32;
type FnStopAcquisition = unsafe extern "C" fn(UnicornHandle) -> i32;
type FnSetConfiguration = unsafe extern "C" fn(UnicornHandle, *mut UnicornAmplifierConfiguration) -> i32;
type FnGetConfiguration = unsafe extern "C" fn(UnicornHandle, *mut UnicornAmplifierConfiguration) -> i32;
type FnGetData = unsafe extern "C" fn(UnicornHandle, u32, *mut f32, u32) -> i32;
type FnGetChannelIndex = unsafe extern "C" fn(UnicornHandle, *const c_char, *mut u32) -> i32;
type FnGetNumberOfAcquiredChannels = unsafe extern "C" fn(UnicornHandle, *mut u32) -> i32;
type FnGetDeviceInformation = unsafe extern "C" fn(UnicornHandle, *mut UnicornDeviceInformation) -> i32;
type FnSetDigitalOutputs = unsafe extern "C" fn(UnicornHandle, u8) -> i32;
type FnGetDigitalOutputs = unsafe extern "C" fn(UnicornHandle, *mut u8) -> i32;

// ── Library wrapper ──────────────────────────────────────────────────────────

/// Dynamically-loaded Unicorn C API with all 16 functions.
pub struct UnicornLib {
    _lib: libloading::Library,

    pub(crate) fn_get_api_version: FnGetApiVersion,
    pub(crate) fn_get_last_error_text: FnGetLastErrorText,
    pub(crate) fn_get_bluetooth_adapter_info: FnGetBluetoothAdapterInfo,
    pub(crate) fn_get_available_devices: FnGetAvailableDevices,
    pub(crate) fn_open_device: FnOpenDevice,
    pub(crate) fn_close_device: FnCloseDevice,
    pub(crate) fn_start_acquisition: FnStartAcquisition,
    pub(crate) fn_stop_acquisition: FnStopAcquisition,
    pub(crate) fn_set_configuration: FnSetConfiguration,
    pub(crate) fn_get_configuration: FnGetConfiguration,
    pub(crate) fn_get_data: FnGetData,
    pub(crate) fn_get_channel_index: FnGetChannelIndex,
    pub(crate) fn_get_number_of_acquired_channels: FnGetNumberOfAcquiredChannels,
    pub(crate) fn_get_device_information: FnGetDeviceInformation,
    pub(crate) fn_set_digital_outputs: FnSetDigitalOutputs,
    pub(crate) fn_get_digital_outputs: FnGetDigitalOutputs,
}

unsafe impl Send for UnicornLib {}
unsafe impl Sync for UnicornLib {}

macro_rules! load_fn {
    ($lib:expr, $name:literal, $ty:ty) => {
        *$lib.get::<$ty>($name).map_err(|e| UnicornError::LibraryNotAvailable {
            reason: format!("{}: {}", std::str::from_utf8($name).unwrap_or("?"), e),
        })?
    };
}

impl UnicornLib {
    /// Load the Unicorn shared library from the system search path.
    fn load() -> Result<Self, UnicornError> {
        // Try platform-specific names
        let lib_names: &[&str] = if cfg!(target_os = "windows") {
            &["Unicorn", "Unicorn.dll"]
        } else {
            &["unicorn", "libunicorn.so"]
        };

        let mut last_err = String::new();
        for name in lib_names {
            let lib_name = libloading::library_filename(name);
            match unsafe { libloading::Library::new(&lib_name) } {
                Ok(lib) => {
                    return unsafe { Self::from_lib(lib) };
                }
                Err(e) => {
                    last_err = format!("{:?}: {}", lib_name, e);
                }
            }
        }

        Err(UnicornError::LibraryNotAvailable {
            reason: format!(
                "Could not load Unicorn library.\nLast attempt: {}\n\
                 Run ./sdk/download.sh to download the official library.",
                last_err
            ),
        })
    }

    unsafe fn from_lib(lib: libloading::Library) -> Result<Self, UnicornError> {
        Ok(UnicornLib {
            fn_get_api_version: load_fn!(lib, b"UNICORN_GetApiVersion\0", FnGetApiVersion),
            fn_get_last_error_text: load_fn!(lib, b"UNICORN_GetLastErrorText\0", FnGetLastErrorText),
            fn_get_bluetooth_adapter_info: load_fn!(lib, b"UNICORN_GetBluetoothAdapterInfo\0", FnGetBluetoothAdapterInfo),
            fn_get_available_devices: load_fn!(lib, b"UNICORN_GetAvailableDevices\0", FnGetAvailableDevices),
            fn_open_device: load_fn!(lib, b"UNICORN_OpenDevice\0", FnOpenDevice),
            fn_close_device: load_fn!(lib, b"UNICORN_CloseDevice\0", FnCloseDevice),
            fn_start_acquisition: load_fn!(lib, b"UNICORN_StartAcquisition\0", FnStartAcquisition),
            fn_stop_acquisition: load_fn!(lib, b"UNICORN_StopAcquisition\0", FnStopAcquisition),
            fn_set_configuration: load_fn!(lib, b"UNICORN_SetConfiguration\0", FnSetConfiguration),
            fn_get_configuration: load_fn!(lib, b"UNICORN_GetConfiguration\0", FnGetConfiguration),
            fn_get_data: load_fn!(lib, b"UNICORN_GetData\0", FnGetData),
            fn_get_channel_index: load_fn!(lib, b"UNICORN_GetChannelIndex\0", FnGetChannelIndex),
            fn_get_number_of_acquired_channels: load_fn!(lib, b"UNICORN_GetNumberOfAcquiredChannels\0", FnGetNumberOfAcquiredChannels),
            fn_get_device_information: load_fn!(lib, b"UNICORN_GetDeviceInformation\0", FnGetDeviceInformation),
            fn_set_digital_outputs: load_fn!(lib, b"UNICORN_SetDigitalOutputs\0", FnSetDigitalOutputs),
            fn_get_digital_outputs: load_fn!(lib, b"UNICORN_GetDigitalOutputs\0", FnGetDigitalOutputs),
            _lib: lib,
        })
    }
}

// ── Singleton accessor ───────────────────────────────────────────────────────

static SDK_LIB: OnceLock<Result<UnicornLib, String>> = OnceLock::new();

/// Get the global Unicorn library handle (loaded once on first call).
pub fn sdk_lib() -> Result<&'static UnicornLib, UnicornError> {
    SDK_LIB
        .get_or_init(|| UnicornLib::load().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| UnicornError::LibraryNotAvailable { reason: e.clone() })
}
