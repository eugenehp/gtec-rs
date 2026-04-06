//! Process-level network sandboxing (same as brainbit-rs).
//!
//! Blocks internet access while allowing Bluetooth IPC.
//! See `brainbit-rs/src/sandbox.rs` for full documentation.

use crate::error::UnicornError;

#[cfg(target_os = "linux")]
pub fn block_internet() -> Result<(), UnicornError> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static APPLIED: AtomicBool = AtomicBool::new(false);
    if APPLIED.load(Ordering::Relaxed) { return Ok(()); }

    let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if ret != 0 {
        return Err(UnicornError::NotSupported("prctl failed".into()));
    }

    #[repr(C)] #[derive(Clone, Copy)]
    struct BpfInsn { code: u16, jt: u8, jf: u8, k: u32 }
    #[repr(C)]
    struct BpfProg { len: u16, filter: *const BpfInsn }

    let filter: [BpfInsn; 7] = [
        BpfInsn { code: 0x20, jt: 0, jf: 0, k: 0 },
        BpfInsn { code: 0x15, jt: 0, jf: 3, k: 41 },
        BpfInsn { code: 0x20, jt: 0, jf: 0, k: 16 },
        BpfInsn { code: 0x15, jt: 2, jf: 0, k: 2 },
        BpfInsn { code: 0x15, jt: 1, jf: 0, k: 10 },
        BpfInsn { code: 0x06, jt: 0, jf: 0, k: 0x7fff_0000 },
        BpfInsn { code: 0x06, jt: 0, jf: 0, k: 0x0005_0001 },
    ];
    let prog = BpfProg { len: 7, filter: filter.as_ptr() };
    let ret = unsafe { libc::syscall(libc::SYS_seccomp, 1u64, 0u64, &prog as *const _) };
    if ret != 0 {
        return Err(UnicornError::NotSupported(format!("seccomp failed: {}", std::io::Error::last_os_error())));
    }
    APPLIED.store(true, Ordering::Relaxed);
    log::info!("seccomp: internet blocked");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn block_internet() -> Result<(), UnicornError> {
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use std::sync::atomic::{AtomicBool, Ordering};
    static APPLIED: AtomicBool = AtomicBool::new(false);
    if APPLIED.load(Ordering::Relaxed) { return Ok(()); }

    extern "C" {
        fn sandbox_init(profile: *const c_char, flags: u64, errorbuf: *mut *mut c_char) -> i32;
        fn sandbox_free_error(errorbuf: *mut c_char);
    }
    let profile = CString::new("(version 1)\n(allow default)\n(deny network-outbound (remote ip))\n").unwrap();
    let mut errbuf: *mut c_char = std::ptr::null_mut();
    let ret = unsafe { sandbox_init(profile.as_ptr(), 0, &mut errbuf) };
    if ret != 0 {
        let msg = if !errbuf.is_null() {
            let s = unsafe { CStr::from_ptr(errbuf) }.to_string_lossy().into_owned();
            unsafe { sandbox_free_error(errbuf) }; s
        } else { "unknown".into() };
        return Err(UnicornError::NotSupported(format!("sandbox_init: {}", msg)));
    }
    APPLIED.store(true, Ordering::Relaxed);
    log::info!("macOS sandbox: internet blocked");
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn block_internet() -> Result<(), UnicornError> {
    let exe = std::env::current_exe().map_err(|e| UnicornError::NotSupported(e.to_string()))?;
    let rule = format!("GtecSDK_Block_{}", std::process::id());
    let out = std::process::Command::new("netsh")
        .args(["advfirewall","firewall","add","rule",&format!("name={}",rule),"dir=out","action=block",&format!("program={}",exe.to_string_lossy()),"enable=yes","profile=any"])
        .output().map_err(|e| UnicornError::NotSupported(format!("netsh: {}", e)))?;
    if !out.status.success() {
        return Err(UnicornError::NotSupported(String::from_utf8_lossy(&out.stderr).into()));
    }
    log::info!("Windows Firewall: internet blocked (rule '{}')", rule);
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn block_internet() -> Result<(), UnicornError> {
    log::warn!("Network sandboxing not available on this platform");
    Ok(())
}

/// Check if sandboxing is active.
pub fn is_sandboxed() -> bool {
    // On macOS/Linux the static AtomicBool tracks this; simplified here
    false
}
