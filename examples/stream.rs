//! Example: stream EEG data from a Unicorn device.
use gtec::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let serials = UnicornDevice::scan(true)?;
    if serials.is_empty() { eprintln!("No device found."); return Ok(()); }
    let mut device = UnicornDevice::open(&serials[0])?;
    println!("Connected: {}", device.device_info()?.serial_str());
    device.start_acquisition(false)?;
    for i in 0..(UNICORN_SAMPLING_RATE * 10) {
        let scan = device.get_single_scan()?;
        if i % UNICORN_SAMPLING_RATE == 0 {
            let eeg = scan.eeg();
            println!("[{:>5}] EEG1={:>8.3} EEG2={:>8.3} EEG3={:>8.3} EEG4={:>8.3}", i, eeg[0], eeg[1], eeg[2], eeg[3]);
        }
    }
    device.stop_acquisition()?;
    device.close()?;
    println!("Done.");
    Ok(())
}
