//! Example: read 4 seconds of EEG data from a Unicorn.
use gtec::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let serials = UnicornDevice::scan(true)?;
    if serials.is_empty() { eprintln!("No device found."); return Ok(()); }
    let mut device = UnicornDevice::open(&serials[0])?;
    println!("Connected: {:?}", device.device_info()?);
    let scans = device.capture(UNICORN_SAMPLING_RATE * 4)?;
    println!("Captured {} scans × {} channels", scans.len(), device.num_acquired_channels());
    device.close()?;
    Ok(())
}
