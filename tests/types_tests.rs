//! Unit tests for FFI types and constants (no hardware required).
use gtec::types::*;

#[test] fn test_sampling_rate() { assert_eq!(UNICORN_SAMPLING_RATE, 250); }
#[test] fn test_eeg_channels() { assert_eq!(UNICORN_EEG_CHANNELS_COUNT, 8); }
#[test] fn test_total_channels() { assert_eq!(UNICORN_TOTAL_CHANNELS_COUNT, 17); }
#[test] fn test_accel_channels() { assert_eq!(UNICORN_ACCELEROMETER_CHANNELS_COUNT, 3); }
#[test] fn test_gyro_channels() { assert_eq!(UNICORN_GYROSCOPE_CHANNELS_COUNT, 3); }
#[test] fn test_digital_outputs() { assert_eq!(UNICORN_NUMBER_OF_DIGITAL_OUTPUTS, 8); }

#[test] fn test_config_indices() {
    assert_eq!(UNICORN_EEG_CONFIG_INDEX, 0);
    assert_eq!(UNICORN_ACCELEROMETER_CONFIG_INDEX, 8);
    assert_eq!(UNICORN_GYROSCOPE_CONFIG_INDEX, 11);
    assert_eq!(UNICORN_BATTERY_CONFIG_INDEX, 14);
    assert_eq!(UNICORN_COUNTER_CONFIG_INDEX, 15);
    assert_eq!(UNICORN_VALIDATION_CONFIG_INDEX, 16);
}

#[test] fn test_error_codes() {
    assert_eq!(UNICORN_ERROR_SUCCESS, 0);
    assert_eq!(UNICORN_ERROR_INVALID_PARAMETER, 1);
    assert_eq!(UNICORN_ERROR_UNSUPPORTED_DEVICE, 10);
    assert_eq!(UNICORN_ERROR_INVALID_HANDLE, -2);
    assert_eq!(UNICORN_ERROR_GENERAL_ERROR, -1);
}

#[test] fn test_error_name() {
    assert_eq!(error_name(0), "Success");
    assert_eq!(error_name(4), "Open device failed");
    assert_eq!(error_name(9), "Connection problem");
    assert_eq!(error_name(999), "Unknown error");
}

#[test] fn test_serial_length() { assert_eq!(UNICORN_SERIAL_LENGTH_MAX, 14); }
#[test] fn test_string_length() { assert_eq!(UNICORN_STRING_LENGTH_MAX, 255); }

#[test] fn test_channel_names() {
    assert_eq!(EEG_CHANNEL_NAMES.len(), 8);
    assert_eq!(EEG_CHANNEL_NAMES[0], "EEG 1");
    assert_eq!(EEG_CHANNEL_NAMES[7], "EEG 8");
    assert_eq!(ALL_CHANNEL_NAMES.len(), 17);
    assert_eq!(ALL_CHANNEL_NAMES[14], "Battery Level");
    assert_eq!(ALL_CHANNEL_NAMES[16], "Validation Indicator");
}

#[test] fn test_amplifier_channel_size() {
    let size = std::mem::size_of::<UnicornAmplifierChannel>();
    // name(32) + unit(32) + range(8) + enabled(4) = 76
    assert_eq!(size, 76);
}

#[test] fn test_amplifier_config_size() {
    let size = std::mem::size_of::<UnicornAmplifierConfiguration>();
    assert_eq!(size, 76 * UNICORN_TOTAL_CHANNELS_COUNT);
}

#[test] fn test_device_info_size() {
    let size = std::mem::size_of::<UnicornDeviceInformation>();
    // numberOfEegChannels(2) + serial(14) + firmware(12) + device(6) + pcb(4) + enclosure(4) = 42
    assert_eq!(size, 42);
}

#[test] fn test_channel_name_str() {
    let mut ch = UnicornAmplifierChannel {
        name: [0u8; 32], unit: [0u8; 32], range: [0.0, 1.0], enabled: 1,
    };
    ch.name[..5].copy_from_slice(b"EEG 1");
    assert_eq!(ch.name_str(), "EEG 1");
    assert!(ch.is_enabled());
    ch.enabled = 0;
    assert!(!ch.is_enabled());
}

#[test] fn test_device_info_str() {
    let mut info = unsafe { std::mem::zeroed::<UnicornDeviceInformation>() };
    info.serial[..6].copy_from_slice(b"UN-001");
    info.firmware_version[..3].copy_from_slice(b"1.5");
    assert_eq!(info.serial_str(), "UN-001");
    assert_eq!(info.firmware_version_str(), "1.5");
}

#[test] fn test_handle_type() {
    assert_eq!(std::mem::size_of::<UnicornHandle>(), 8);
}
