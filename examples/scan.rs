//! Example: scan for Unicorn devices.
use gtec::prelude::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("API version: {}", UnicornDevice::api_version()?);
    println!("Scanning for paired devices...");
    let serials = UnicornDevice::scan(true)?;
    println!("Found {} device(s):", serials.len());
    for s in &serials { println!("  {}", s); }
    Ok(())
}
