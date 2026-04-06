//! Pure-Rust BLE backend for the Unicorn BCI Core-8.
//!
//! Uses `btleplug` for cross-platform BLE — works on **Linux**, **macOS**, and
//! **Windows** without any native g.tec library.
//!
//! The GATT service and characteristic UUIDs were extracted from g.tec's official
//! `libgtecble.dylib` (from `gtec-ble` PyPI package, used by `gpype` SDK).
//!
//! # GATT Profile
//!
//! | UUID | Type | Purpose |
//! |---|---|---|
//! | `39a76676-2788-46c9-afa0-f0c0c31e6fd9` | Service | Custom g.tec EEG service |
//! | `B5211405-...` | Notify | EEG data stream (subscribe) |
//! | `B5211406-...` | Write | Control commands (start/stop) |
//! | `B5211408-...` | Read | Payload configuration |
//! | `B5211409-...` | Read | Channel configuration |
//! | `B521140A-...` | Read | Filter configuration |
//! | `0x180A` | Service | Device Information (standard) |
//! | `0x180F` / `0x2A19` | Service/Char | Battery Level (standard) |
//!
//! # Example
//!
//! ```rust,ignore
//! use gtec::ble::BleDevice;
//!
//! let devices = BleDevice::scan(std::time::Duration::from_secs(5)).await?;
//! let mut device = BleDevice::connect(&devices[0]).await?;
//! let info = device.device_info().await?;
//! println!("Model: {}, Serial: {}", info.model, info.serial);
//!
//! device.start_streaming(|payload| {
//!     println!("EEG: {:?}", payload.eeg);
//! }).await?;
//!
//! tokio::time::sleep(std::time::Duration::from_secs(10)).await;
//! device.stop_streaming().await?;
//! ```

#[cfg(feature = "ble")]
mod inner {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use btleplug::api::{
        Central, CentralEvent, Characteristic, Manager as _, Peripheral as _, ScanFilter,
        WriteType,
    };
    use btleplug::platform::{Adapter, Manager, Peripheral};
    use tokio::sync::mpsc;
    use uuid::Uuid;

    use crate::error::UnicornError;
    use crate::protocol::{self, UnicornPayload, PAYLOAD_LENGTH};

    // ── GATT UUIDs (extracted from libgtecble.dylib) ─────────────────────

    /// Custom g.tec EEG service UUID.
    pub const SERVICE_UUID: Uuid = Uuid::from_fields(
        0x39a76676, 0x2788, 0x46c9,
        &[0xaf, 0xa0, 0xf0, 0xc0, 0xc3, 0x1e, 0x6f, 0xd9],
    );

    /// Data notification characteristic (subscribe for EEG payloads).
    pub const CHAR_DATA_NOTIFY: Uuid = Uuid::from_fields(
        0xB5211405, 0x449F, 0x4BA0,
        &[0xA6, 0x51, 0xB8, 0x87, 0x92, 0x4B, 0x81, 0xE8],
    );

    /// Control/write characteristic (start/stop commands).
    pub const CHAR_CONTROL_WRITE: Uuid = Uuid::from_fields(
        0xB5211406, 0x449F, 0x4BA0,
        &[0xA6, 0x51, 0xB8, 0x87, 0x92, 0x4B, 0x81, 0xE8],
    );

    /// Payload configuration characteristic (read).
    pub const CHAR_PAYLOAD_CONFIG: Uuid = Uuid::from_fields(
        0xB5211408, 0x449F, 0x4BA0,
        &[0xA6, 0x51, 0xB8, 0x87, 0x92, 0x4B, 0x81, 0xE8],
    );

    /// Channel configuration characteristic (read).
    pub const CHAR_CHANNEL_CONFIG: Uuid = Uuid::from_fields(
        0xB5211409, 0x449F, 0x4BA0,
        &[0xA6, 0x51, 0xB8, 0x87, 0x92, 0x4B, 0x81, 0xE8],
    );

    /// Filter configuration characteristic (read).
    pub const CHAR_FILTER_CONFIG: Uuid = Uuid::from_fields(
        0xB521140A, 0x449F, 0x4BA0,
        &[0xA6, 0x51, 0xB8, 0x87, 0x92, 0x4B, 0x81, 0xE8],
    );

    /// Standard Battery Level characteristic.
    pub const CHAR_BATTERY_LEVEL: Uuid = Uuid::from_u16(0x2A19);

    // ── Discovered device info ───────────────────────────────────────────

    /// A discovered BCI Core-8 device.
    #[derive(Debug, Clone)]
    pub struct DiscoveredDevice {
        /// Device name from BLE advertisement.
        pub name: String,
        /// BLE address / identifier.
        pub address: String,
        /// RSSI signal strength.
        pub rssi: Option<i16>,
        /// Internal peripheral reference.
        peripheral: Peripheral,
    }

    /// Device information read from GATT characteristics.
    #[derive(Debug, Clone)]
    pub struct BleDeviceInfo {
        pub model: String,
        pub serial: String,
        pub firmware: String,
        pub hardware: String,
        pub manufacturer: String,
    }

    // ── BLE Device ───────────────────────────────────────────────────────

    /// A connected BCI Core-8 device over BLE.
    pub struct BleDevice {
        peripheral: Peripheral,
        data_char: Option<Characteristic>,
        control_char: Option<Characteristic>,
    }

    impl BleDevice {
        /// Scan for BCI Core-8 devices for the given duration.
        pub async fn scan(timeout: Duration) -> Result<Vec<DiscoveredDevice>, UnicornError> {
            let manager = Manager::new().await.map_err(|e| {
                UnicornError::LibraryNotAvailable {
                    reason: format!("BLE manager init failed: {}", e),
                }
            })?;

            let adapters = manager.adapters().await.map_err(|e| {
                UnicornError::LibraryNotAvailable {
                    reason: format!("No BLE adapter: {}", e),
                }
            })?;

            let adapter = adapters.into_iter().next().ok_or(UnicornError::LibraryNotAvailable {
                reason: "No BLE adapter found".into(),
            })?;

            // Scan with filter for our service UUID
            adapter
                .start_scan(ScanFilter {
                    services: vec![SERVICE_UUID],
                })
                .await
                .map_err(|e| UnicornError::SdkError {
                    code: -1,
                    message: format!("Scan failed: {}", e),
                })?;

            tokio::time::sleep(timeout).await;
            adapter.stop_scan().await.ok();

            let peripherals = adapter.peripherals().await.map_err(|e| {
                UnicornError::SdkError {
                    code: -1,
                    message: format!("Failed to list peripherals: {}", e),
                }
            })?;

            let mut devices = Vec::new();
            for p in peripherals {
                if let Some(props) = p.properties().await.ok().flatten() {
                    if props.services.contains(&SERVICE_UUID) {
                        devices.push(DiscoveredDevice {
                            name: props.local_name.unwrap_or_else(|| "Unknown".into()),
                            address: props.address.to_string(),
                            rssi: props.rssi,
                            peripheral: p,
                        });
                    }
                }
            }

            Ok(devices)
        }

        /// Connect to a discovered device.
        pub async fn connect(discovered: &DiscoveredDevice) -> Result<Self, UnicornError> {
            let p = &discovered.peripheral;

            p.connect().await.map_err(|e| UnicornError::SdkError {
                code: -1,
                message: format!("Connect failed: {}", e),
            })?;

            p.discover_services().await.map_err(|e| UnicornError::SdkError {
                code: -1,
                message: format!("Service discovery failed: {}", e),
            })?;

            let chars = p.characteristics();
            let data_char = chars.iter().find(|c| c.uuid == CHAR_DATA_NOTIFY).cloned();
            let control_char = chars.iter().find(|c| c.uuid == CHAR_CONTROL_WRITE).cloned();

            Ok(BleDevice {
                peripheral: discovered.peripheral.clone(),
                data_char,
                control_char,
            })
        }

        /// Read device information from standard BLE services.
        pub async fn device_info(&self) -> Result<BleDeviceInfo, UnicornError> {
            // Read from Device Information Service (0x180A) characteristics
            let chars = self.peripheral.characteristics();

            let read_char = |uuid: Uuid| -> String {
                // This is a simplification — in practice you'd read from the peripheral
                chars
                    .iter()
                    .find(|c| c.uuid == uuid)
                    .map(|_| "".to_string())
                    .unwrap_or_default()
            };

            // Standard Device Information Service UUIDs
            let model = self.read_characteristic_string(Uuid::from_u16(0x2A24)).await;
            let serial = self.read_characteristic_string(Uuid::from_u16(0x2A25)).await;
            let firmware = self.read_characteristic_string(Uuid::from_u16(0x2A26)).await;
            let hardware = self.read_characteristic_string(Uuid::from_u16(0x2A27)).await;
            let manufacturer = self.read_characteristic_string(Uuid::from_u16(0x2A29)).await;

            Ok(BleDeviceInfo { model, serial, firmware, hardware, manufacturer })
        }

        async fn read_characteristic_string(&self, uuid: Uuid) -> String {
            let chars = self.peripheral.characteristics();
            if let Some(c) = chars.iter().find(|c| c.uuid == uuid) {
                if let Ok(data) = self.peripheral.read(c).await {
                    return String::from_utf8_lossy(&data).trim_end_matches('\0').to_string();
                }
            }
            String::new()
        }

        /// Read the battery level (0-100%).
        pub async fn battery_level(&self) -> Result<u8, UnicornError> {
            let chars = self.peripheral.characteristics();
            let c = chars.iter().find(|c| c.uuid == CHAR_BATTERY_LEVEL).ok_or(
                UnicornError::NotSupported("Battery characteristic not found".into()),
            )?;
            let data = self.peripheral.read(c).await.map_err(|e| UnicornError::SdkError {
                code: -1,
                message: format!("Read battery failed: {}", e),
            })?;
            Ok(*data.first().unwrap_or(&0))
        }

        /// Start streaming EEG data. Calls the callback with decoded payloads.
        ///
        /// Uses the same 45-byte payload format as the Hybrid Black (decoded
        /// by `protocol::decode_payload`).
        pub async fn start_streaming<F>(
            &self,
            mut callback: F,
        ) -> Result<(), UnicornError>
        where
            F: FnMut(UnicornPayload) + Send + 'static,
        {
            let data_char = self.data_char.as_ref().ok_or(
                UnicornError::NotSupported("Data characteristic not found".into()),
            )?;

            // Subscribe to notifications
            self.peripheral.subscribe(data_char).await.map_err(|e| {
                UnicornError::SdkError {
                    code: -1,
                    message: format!("Subscribe failed: {}", e),
                }
            })?;

            // Send start acquisition command
            if let Some(ref ctrl) = self.control_char {
                self.peripheral
                    .write(ctrl, &protocol::CMD_START_ACQUISITION, WriteType::WithResponse)
                    .await
                    .map_err(|e| UnicornError::SdkError {
                        code: -1,
                        message: format!("Start command failed: {}", e),
                    })?;
            }

            // Spawn a task to handle notifications
            let mut notifications = self.peripheral.notifications().await.map_err(|e| {
                UnicornError::SdkError {
                    code: -1,
                    message: format!("Notification stream failed: {}", e),
                }
            })?;

            tokio::spawn(async move {
                use tokio_stream::StreamExt;
                while let Some(notification) = notifications.next().await {
                    if notification.uuid == CHAR_DATA_NOTIFY
                        && notification.value.len() == PAYLOAD_LENGTH
                    {
                        let mut buf = [0u8; PAYLOAD_LENGTH];
                        buf.copy_from_slice(&notification.value);
                        if let Some(payload) = protocol::decode_payload(&buf) {
                            callback(payload);
                        }
                    }
                }
            });

            Ok(())
        }

        /// Stop streaming EEG data.
        pub async fn stop_streaming(&self) -> Result<(), UnicornError> {
            // Send stop command
            if let Some(ref ctrl) = self.control_char {
                self.peripheral
                    .write(ctrl, &protocol::CMD_STOP_ACQUISITION, WriteType::WithResponse)
                    .await
                    .ok(); // Don't fail if already stopped
            }

            // Unsubscribe from notifications
            if let Some(ref data) = self.data_char {
                self.peripheral.unsubscribe(data).await.ok();
            }

            Ok(())
        }

        /// Read the channel configuration from the device.
        pub async fn channel_config(&self) -> Result<Vec<u8>, UnicornError> {
            self.read_config_char(CHAR_CHANNEL_CONFIG).await
        }

        /// Read the payload configuration from the device.
        pub async fn payload_config(&self) -> Result<Vec<u8>, UnicornError> {
            self.read_config_char(CHAR_PAYLOAD_CONFIG).await
        }

        /// Read the filter configuration from the device.
        pub async fn filter_config(&self) -> Result<Vec<u8>, UnicornError> {
            self.read_config_char(CHAR_FILTER_CONFIG).await
        }

        async fn read_config_char(&self, uuid: Uuid) -> Result<Vec<u8>, UnicornError> {
            let chars = self.peripheral.characteristics();
            let c = chars.iter().find(|c| c.uuid == uuid).ok_or(
                UnicornError::NotSupported(format!("Characteristic {} not found", uuid)),
            )?;
            self.peripheral.read(c).await.map_err(|e| UnicornError::SdkError {
                code: -1,
                message: format!("Read failed: {}", e),
            })
        }

        /// Disconnect from the device.
        pub async fn disconnect(&self) -> Result<(), UnicornError> {
            self.peripheral.disconnect().await.map_err(|e| UnicornError::SdkError {
                code: -1,
                message: format!("Disconnect failed: {}", e),
            })
        }
    }
}

#[cfg(feature = "ble")]
pub use inner::*;
