//! High-level device abstraction for the Unicorn Hybrid Black.
//!
//! ```rust,ignore
//! use gtec::prelude::*;
//!
//! let serials = UnicornDevice::scan(true)?;
//! let mut device = UnicornDevice::open(&serials[0])?;
//!
//! println!("Info: {:?}", device.device_info()?);
//! println!("Battery: {}%", device.battery_level()?);
//!
//! let data = device.capture(UNICORN_SAMPLING_RATE * 4)?;
//! device.close()?;
//! ```

use std::ffi::{CStr, CString};

use crate::error::UnicornError;
use crate::ffi::sdk_lib;
use crate::types::*;

/// Check a Unicorn API return code and convert to Result.
fn check(code: i32) -> Result<(), UnicornError> {
    if code == UNICORN_ERROR_SUCCESS {
        Ok(())
    } else {
        let msg = last_error_text().unwrap_or_else(|| error_name(code).to_string());
        Err(UnicornError::SdkError { code, message: msg })
    }
}

/// Get the last error text from the SDK.
fn last_error_text() -> Option<String> {
    let lib = sdk_lib().ok()?;
    let ptr = unsafe { (lib.fn_get_last_error_text)() };
    if ptr.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
    if s.is_empty() { None } else { Some(s) }
}

/// A single scan of all acquired channels.
#[derive(Debug, Clone)]
pub struct Scan {
    /// Raw f32 values for each acquired channel in this scan.
    pub data: Vec<f32>,
}

impl Scan {
    /// Get EEG values (first 8 channels by default).
    pub fn eeg(&self) -> &[f32] {
        let end = UNICORN_EEG_CHANNELS_COUNT.min(self.data.len());
        &self.data[..end]
    }
}

/// A connected Unicorn Hybrid Black device.
pub struct UnicornDevice {
    handle: UnicornHandle,
    num_acquired_channels: u32,
    acquiring: bool,
}

impl UnicornDevice {
    // ── Static methods ───────────────────────────────────────────────────

    /// Get the API version.
    pub fn api_version() -> Result<f32, UnicornError> {
        let lib = sdk_lib()?;
        Ok(unsafe { (lib.fn_get_api_version)() })
    }

    /// Get Bluetooth adapter info (Windows only).
    pub fn bluetooth_adapter_info() -> Result<UnicornBluetoothAdapterInfo, UnicornError> {
        let lib = sdk_lib()?;
        let mut info = unsafe { std::mem::zeroed::<UnicornBluetoothAdapterInfo>() };
        check(unsafe { (lib.fn_get_bluetooth_adapter_info)(&mut info) })?;
        Ok(info)
    }

    /// Scan for available Unicorn devices.
    ///
    /// If `only_paired` is `true`, returns only paired devices (fast).
    /// If `false`, performs an extensive scan for unpaired devices (slow).
    ///
    /// Returns a list of serial number strings.
    pub fn scan(only_paired: bool) -> Result<Vec<String>, UnicornError> {
        let lib = sdk_lib()?;

        // First call to get count
        let mut count: u32 = 0;
        let paired_flag: UnicornBool = if only_paired { 1 } else { 0 };
        check(unsafe {
            (lib.fn_get_available_devices)(std::ptr::null_mut(), &mut count, paired_flag)
        })?;

        if count == 0 {
            return Ok(Vec::new());
        }

        // Second call to get serials
        let mut serials: Vec<UnicornDeviceSerial> = vec![[0u8; UNICORN_SERIAL_LENGTH_MAX]; count as usize];
        check(unsafe {
            (lib.fn_get_available_devices)(serials.as_mut_ptr(), &mut count, paired_flag)
        })?;

        serials.truncate(count as usize);
        let result = serials
            .iter()
            .map(|s| {
                let nul = s.iter().position(|&b| b == 0).unwrap_or(UNICORN_SERIAL_LENGTH_MAX);
                String::from_utf8_lossy(&s[..nul]).into_owned()
            })
            .collect();

        Ok(result)
    }

    /// Open a device by serial number.
    pub fn open(serial: &str) -> Result<Self, UnicornError> {
        let lib = sdk_lib()?;
        let c_serial = CString::new(serial).map_err(|_| UnicornError::SdkError {
            code: UNICORN_ERROR_INVALID_PARAMETER,
            message: "Invalid serial string".into(),
        })?;

        let mut handle: UnicornHandle = 0;
        check(unsafe { (lib.fn_open_device)(c_serial.as_ptr(), &mut handle) })?;

        // Get number of acquired channels
        let mut num_channels: u32 = 0;
        check(unsafe { (lib.fn_get_number_of_acquired_channels)(handle, &mut num_channels) })?;

        Ok(UnicornDevice {
            handle,
            num_acquired_channels: num_channels,
            acquiring: false,
        })
    }

    // ── Instance methods ─────────────────────────────────────────────────

    /// Close the device connection.
    pub fn close(&mut self) -> Result<(), UnicornError> {
        if self.acquiring {
            let _ = self.stop_acquisition();
        }
        let lib = sdk_lib()?;
        check(unsafe { (lib.fn_close_device)(&mut self.handle) })
    }

    /// Get device information (serial, firmware version, etc.).
    pub fn device_info(&self) -> Result<UnicornDeviceInformation, UnicornError> {
        let lib = sdk_lib()?;
        let mut info = unsafe { std::mem::zeroed::<UnicornDeviceInformation>() };
        check(unsafe { (lib.fn_get_device_information)(self.handle, &mut info) })?;
        Ok(info)
    }

    /// Get the current amplifier configuration.
    pub fn configuration(&self) -> Result<UnicornAmplifierConfiguration, UnicornError> {
        let lib = sdk_lib()?;
        let mut config = unsafe { std::mem::zeroed::<UnicornAmplifierConfiguration>() };
        check(unsafe { (lib.fn_get_configuration)(self.handle, &mut config) })?;
        Ok(config)
    }

    /// Set the amplifier configuration.
    pub fn set_configuration(&self, config: &mut UnicornAmplifierConfiguration) -> Result<(), UnicornError> {
        let lib = sdk_lib()?;
        check(unsafe { (lib.fn_set_configuration)(self.handle, config) })
    }

    /// Get the number of currently acquired channels.
    pub fn num_acquired_channels(&self) -> u32 {
        self.num_acquired_channels
    }

    /// Get the index of a channel by name within an acquired scan.
    pub fn channel_index(&self, name: &str) -> Result<u32, UnicornError> {
        let lib = sdk_lib()?;
        let c_name = CString::new(name).map_err(|_| UnicornError::SdkError {
            code: UNICORN_ERROR_INVALID_PARAMETER,
            message: "Invalid channel name".into(),
        })?;
        let mut index: u32 = 0;
        check(unsafe { (lib.fn_get_channel_index)(self.handle, c_name.as_ptr(), &mut index) })?;
        Ok(index)
    }

    /// Start data acquisition.
    ///
    /// If `test_signal` is `true`, the device outputs a test signal instead
    /// of real EEG data (useful for verifying connectivity).
    pub fn start_acquisition(&mut self, test_signal: bool) -> Result<(), UnicornError> {
        let lib = sdk_lib()?;
        let flag: UnicornBool = if test_signal { 1 } else { 0 };
        check(unsafe { (lib.fn_start_acquisition)(self.handle, flag) })?;

        // Refresh channel count
        let mut n: u32 = 0;
        check(unsafe { (lib.fn_get_number_of_acquired_channels)(self.handle, &mut n) })?;
        self.num_acquired_channels = n;
        self.acquiring = true;
        Ok(())
    }

    /// Stop data acquisition.
    pub fn stop_acquisition(&mut self) -> Result<(), UnicornError> {
        let lib = sdk_lib()?;
        check(unsafe { (lib.fn_stop_acquisition)(self.handle) })?;
        self.acquiring = false;
        Ok(())
    }

    /// Whether the device is currently acquiring data.
    pub fn is_acquiring(&self) -> bool {
        self.acquiring
    }

    /// Read `n_scans` of data from the device.
    ///
    /// Each scan contains one f32 per acquired channel.
    /// The device must be in acquisition mode.
    pub fn get_data(&self, n_scans: u32) -> Result<Vec<Scan>, UnicornError> {
        let lib = sdk_lib()?;
        let n_ch = self.num_acquired_channels;
        let buf_len = n_scans * n_ch;
        let mut buffer: Vec<f32> = vec![0.0; buf_len as usize];

        check(unsafe {
            (lib.fn_get_data)(self.handle, n_scans, buffer.as_mut_ptr(), buf_len)
        })?;

        let scans = buffer
            .chunks(n_ch as usize)
            .map(|chunk| Scan { data: chunk.to_vec() })
            .collect();

        Ok(scans)
    }

    /// Read a single scan of data.
    pub fn get_single_scan(&self) -> Result<Scan, UnicornError> {
        let mut scans = self.get_data(1)?;
        scans.pop().ok_or(UnicornError::SdkError {
            code: UNICORN_ERROR_BUFFER_UNDERFLOW,
            message: "No data returned".into(),
        })
    }

    /// Set digital output states (8 bits, one per output).
    pub fn set_digital_outputs(&self, outputs: u8) -> Result<(), UnicornError> {
        let lib = sdk_lib()?;
        check(unsafe { (lib.fn_set_digital_outputs)(self.handle, outputs) })
    }

    /// Get digital output states (8 bits, one per output).
    pub fn get_digital_outputs(&self) -> Result<u8, UnicornError> {
        let lib = sdk_lib()?;
        let mut outputs: u8 = 0;
        check(unsafe { (lib.fn_get_digital_outputs)(self.handle, &mut outputs) })?;
        Ok(outputs)
    }

    // ── Convenience methods ──────────────────────────────────────────────

    /// Capture `n_scans` of EEG data (blocking convenience method).
    ///
    /// Starts acquisition in measurement mode, reads data, stops acquisition.
    pub fn capture(&mut self, n_scans: u32) -> Result<Vec<Scan>, UnicornError> {
        self.start_acquisition(false)?;

        let mut all_scans = Vec::with_capacity(n_scans as usize);
        let mut remaining = n_scans;
        let batch = 250u32; // read 1 second at a time

        while remaining > 0 {
            let to_read = remaining.min(batch);
            let scans = self.get_data(to_read)?;
            all_scans.extend(scans);
            remaining -= to_read;
        }

        self.stop_acquisition()?;
        Ok(all_scans)
    }

    /// Read the battery level (convenience — gets it from a single scan).
    pub fn battery_level(&mut self) -> Result<f32, UnicornError> {
        let idx = self.channel_index("Battery Level")? as usize;
        self.start_acquisition(false)?;
        let scan = self.get_single_scan()?;
        self.stop_acquisition()?;
        Ok(*scan.data.get(idx).unwrap_or(&0.0))
    }
}

impl Drop for UnicornDevice {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
