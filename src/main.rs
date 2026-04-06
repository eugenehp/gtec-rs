//! CLI: scan for Unicorn devices, connect, and stream EEG data.

use gtec::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("g.tec Unicorn Hybrid Black — Rust CLI");
    println!("=====================================\n");
    println!("API version: {}", UnicornDevice::api_version()?);

    println!("\nScanning for paired Unicorn devices...");
    let serials = UnicornDevice::scan(true)?;
    if serials.is_empty() {
        eprintln!("No Unicorn device found. Make sure the device is paired and powered on.");
        return Ok(());
    }

    println!("Found {} device(s):", serials.len());
    for (i, s) in serials.iter().enumerate() {
        println!("  [{}] {}", i, s);
    }

    println!("\nConnecting to {}...", serials[0]);
    let mut device = UnicornDevice::open(&serials[0])?;

    let info = device.device_info()?;
    println!("  Serial:    {}", info.serial_str());
    println!("  Firmware:  {}", info.firmware_version_str());
    println!("  Version:   {}", info.device_version_str());
    println!("  EEG Ch:    {}", info.number_of_eeg_channels);
    println!("  Channels:  {}", device.num_acquired_channels());

    let n = UNICORN_SAMPLING_RATE * 4;
    println!("\nCapturing {} scans (~4 seconds)...", n);
    let scans = device.capture(n)?;

    println!("\nFirst 10 scans (EEG channels in µV):");
    println!("{:>8} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Scan#", "EEG1", "EEG2", "EEG3", "EEG4", "EEG5", "EEG6", "EEG7", "EEG8");
    for (i, s) in scans.iter().take(10).enumerate() {
        let eeg = s.eeg();
        println!("{:>8} {:>10.3} {:>10.3} {:>10.3} {:>10.3} {:>10.3} {:>10.3} {:>10.3} {:>10.3}",
            i, eeg[0], eeg[1], eeg[2], eeg[3], eeg[4], eeg[5], eeg[6], eeg[7]);
    }

    println!("\nDone — {} scans × {} channels.", scans.len(), device.num_acquired_channels());
    device.close()?;
    Ok(())
}
