//! Unicorn Hybrid Black Bluetooth protocol implementation.
//!
//! This is a pure-Rust implementation of the Unicorn Bluetooth Protocol
//! as documented in `UnicornBluetoothProtocol.pdf` from g.tec.
//!
//! The device uses **classic Bluetooth RFCOMM** (not BLE). Each payload
//! is 45 bytes at 250 Hz:
//!
//! ```text
//! [0..1]   Start:  0xC0 0x00
//! [2]      Battery (4-bit voltage in low nibble)
//! [3..26]  EEG 1-8 (3 bytes each, 24-bit signed big-endian)
//! [27..32] Accelerometer X/Y/Z (2 bytes each, 16-bit signed little-endian)
//! [33..38] Gyroscope X/Y/Z (2 bytes each, 16-bit signed little-endian)
//! [39..42] Counter (4 bytes, 32-bit unsigned little-endian)
//! [43..44] Stop: 0x0D 0x0A
//! ```

/// Payload size in bytes.
pub const PAYLOAD_LENGTH: usize = 45;

/// Start acquisition command (3 bytes).
pub const CMD_START_ACQUISITION: [u8; 3] = [0x61, 0x7C, 0x87];

/// Stop acquisition command (3 bytes).
pub const CMD_STOP_ACQUISITION: [u8; 3] = [0x63, 0x5C, 0xC5];

/// Expected acknowledge response (3 bytes of zeros).
pub const ACK_RESPONSE: [u8; 3] = [0x00, 0x00, 0x00];

/// Payload header.
pub const PAYLOAD_HEADER: [u8; 2] = [0xC0, 0x00];

/// Payload footer.
pub const PAYLOAD_FOOTER: [u8; 2] = [0x0D, 0x0A];

/// EEG scale factor: channelValue * 4500000 / 50331642 = µV
pub const EEG_SCALE_FACTOR: f32 = 4_500_000.0 / 50_331_642.0;

/// Accelerometer scale factor: raw / 4096 = g
pub const ACCEL_SCALE_FACTOR: f32 = 1.0 / 4096.0;

/// Gyroscope scale factor: raw / 32.8 = °/s
pub const GYRO_SCALE_FACTOR: f32 = 1.0 / 32.8;

/// A decoded payload from the Unicorn device.
#[derive(Debug, Clone)]
pub struct UnicornPayload {
    /// Battery level (0-100%).
    pub battery_percent: f32,
    /// 8 EEG channels in µV.
    pub eeg: [f32; 8],
    /// Accelerometer [X, Y, Z] in g.
    pub accelerometer: [f32; 3],
    /// Gyroscope [X, Y, Z] in °/s.
    pub gyroscope: [f32; 3],
    /// Sample counter.
    pub counter: u32,
}

/// Decode a 45-byte raw payload into physical values.
///
/// Returns `None` if the header/footer don't match.
pub fn decode_payload(raw: &[u8; PAYLOAD_LENGTH]) -> Option<UnicornPayload> {
    // Validate header and footer
    if raw[0] != PAYLOAD_HEADER[0] || raw[1] != PAYLOAD_HEADER[1] {
        return None;
    }
    if raw[43] != PAYLOAD_FOOTER[0] || raw[44] != PAYLOAD_FOOTER[1] {
        return None;
    }

    // Battery: bits [3:0] of byte 2
    let battery_raw = (raw[2] & 0x0F) as f32;
    let battery_percent = (100.0 / 1.3) * (battery_raw * 1.3 / 15.0);

    // EEG: 8 channels, 3 bytes each (24-bit signed big-endian), starting at byte 3
    let mut eeg = [0.0f32; 8];
    for i in 0..8 {
        let offset = 3 + i * 3;
        let mut value = (raw[offset] as i32) << 16
            | (raw[offset + 1] as i32) << 8
            | (raw[offset + 2] as i32);

        // Sign extension: if bit 23 is set, extend to 32-bit
        if value & 0x0080_0000 != 0 {
            value |= 0xFF00_0000u32 as i32;
        }

        eeg[i] = value as f32 * EEG_SCALE_FACTOR;
    }

    // Accelerometer: 3 channels, 2 bytes each (16-bit signed little-endian), starting at byte 27
    let mut accelerometer = [0.0f32; 3];
    for i in 0..3 {
        let offset = 27 + i * 2;
        let value = (raw[offset] as u16 | (raw[offset + 1] as u16) << 8) as i16;
        accelerometer[i] = value as f32 * ACCEL_SCALE_FACTOR;
    }

    // Gyroscope: 3 channels, 2 bytes each (16-bit signed little-endian), starting at byte 33
    let mut gyroscope = [0.0f32; 3];
    for i in 0..3 {
        let offset = 33 + i * 2;
        let value = (raw[offset] as u16 | (raw[offset + 1] as u16) << 8) as i16;
        gyroscope[i] = value as f32 * GYRO_SCALE_FACTOR;
    }

    // Counter: 4 bytes little-endian starting at byte 39
    let counter = raw[39] as u32
        | (raw[40] as u32) << 8
        | (raw[41] as u32) << 16
        | (raw[42] as u32) << 24;

    Some(UnicornPayload {
        battery_percent,
        eeg,
        accelerometer,
        gyroscope,
        counter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference payload from the official protocol documentation.
    const EXAMPLE_PAYLOAD: [u8; 45] = [
        0xC0, 0x00, 0x0F, 0x00, 0x9F, 0xAF, 0x00, 0x9F,
        0xD4, 0x00, 0xA0, 0x40, 0x00, 0x9F, 0x43, 0x00,
        0x9F, 0x9A, 0x00, 0x9F, 0xE3, 0x00, 0x9F, 0x85,
        0x00, 0x9F, 0xBB, 0x2E, 0xF6, 0xE9, 0x02, 0x8D,
        0xF2, 0xF3, 0xFF, 0xEF, 0xFF, 0x23, 0x00, 0xB0,
        0x00, 0x00, 0x00, 0x0D, 0x0A,
    ];

    #[test]
    fn test_decode_example_payload() {
        let p = decode_payload(&EXAMPLE_PAYLOAD).expect("should decode");

        // Battery: 100% (0x0F = 15, 15/15 * 100 = 100)
        assert!((p.battery_percent - 100.0).abs() < 0.1);

        // EEG CH1: 0x009FAF → sign-extended → positive
        // 0x009FAF = 40879 → 40879 * 4500000/50331642 ≈ 3654.87 µV
        assert!((p.eeg[0] - 3654.87).abs() < 0.1, "CH1: {}", p.eeg[0]);
        assert!((p.eeg[1] - 3658.18).abs() < 0.1, "CH2: {}", p.eeg[1]);

        // Accelerometer X: 0xF62E (LE) = 0x2EF6 → but LE means raw[27]=0x2E, raw[28]=0xF6
        // 0x2E | 0xF6<<8 = 0xF62E = -2514 as i16 → -2514/4096 ≈ -0.614g
        assert!((p.accelerometer[0] - (-0.614)).abs() < 0.01, "AccX: {}", p.accelerometer[0]);

        // Gyroscope X: raw[33]=0xF3, raw[34]=0xFF → 0xFFF3 = -13 as i16 → -13/32.8 ≈ -0.396
        assert!((p.gyroscope[0] - (-0.397)).abs() < 0.01, "GyrX: {}", p.gyroscope[0]);

        // Counter: 0xB0 0x00 0x00 0x00 LE = 176
        assert_eq!(p.counter, 176);
    }

    #[test]
    fn test_invalid_header() {
        let mut bad = EXAMPLE_PAYLOAD;
        bad[0] = 0xFF;
        assert!(decode_payload(&bad).is_none());
    }

    #[test]
    fn test_invalid_footer() {
        let mut bad = EXAMPLE_PAYLOAD;
        bad[43] = 0xFF;
        assert!(decode_payload(&bad).is_none());
    }

    #[test]
    fn test_eeg_negative_value() {
        let mut payload = [0u8; 45];
        payload[0] = 0xC0; payload[1] = 0x00; // header
        payload[43] = 0x0D; payload[44] = 0x0A; // footer

        // Set CH1 = 0xFF0000 (negative: sign bit set at bit 23)
        payload[3] = 0xFF; payload[4] = 0x00; payload[5] = 0x00;

        let p = decode_payload(&payload).expect("decode");
        // 0xFF0000 | 0xFF000000 = 0xFFFF0000 = -65536 as i32
        // -65536 * 4500000/50331642 ≈ -5861.08 µV
        assert!(p.eeg[0] < 0.0, "Should be negative: {}", p.eeg[0]);
    }

    #[test]
    fn test_constants() {
        assert_eq!(CMD_START_ACQUISITION, [0x61, 0x7C, 0x87]);
        assert_eq!(CMD_STOP_ACQUISITION, [0x63, 0x5C, 0xC5]);
        assert_eq!(ACK_RESPONSE, [0x00, 0x00, 0x00]);
        assert_eq!(PAYLOAD_LENGTH, 45);
    }
}
