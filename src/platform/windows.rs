#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

use crate::error::Result;
use anyhow::bail;

#[cfg(windows)]
pub(crate) fn current_session_id() -> Result<u32> {
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;

    let mut session_id = 0u32;
    // SAFETY: The process ID is obtained from Windows, and `session_id` is a valid
    // writable pointer for the documented output value.
    let ok = unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id) };
    if ok == 0 {
        // SAFETY: GetLastError has no preconditions and reads the thread-local error.
        bail!("ProcessIdToSessionId failed: Windows error {}", unsafe {
            GetLastError()
        });
    }
    Ok(session_id)
}

#[cfg(not(windows))]
pub(crate) fn current_session_id() -> Result<u32> {
    bail!("session IDs are only available on Windows")
}

pub(crate) fn admin_session_ids(current: u32) -> Vec<u32> {
    let mut ids = vec![current];
    if let Ok(entries) = std::fs::read_dir(r"\\.\pipe\") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let lower_name = name.to_ascii_lowercase();
            if let Some(id) = lower_name
                .strip_prefix("predatorsense_admin_agent_")
                .and_then(|raw| raw.parse::<u32>().ok())
                && !ids.contains(&id)
            {
                ids.push(id);
            }
        }
    }
    ids
}

#[cfg(windows)]
pub(crate) fn read_sticky_keys() -> Result<bool> {
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETSTICKYKEYS, SystemParametersInfoW};

    const SKF_STICKYKEYSON: u32 = 0x0000_0001;
    const SKF_HOTKEYACTIVE: u32 = 0x0000_0004;

    #[repr(C)]
    struct StickyKeys {
        cb_size: u32,
        flags: u32,
    }

    let mut state = StickyKeys {
        cb_size: std::mem::size_of::<StickyKeys>() as u32,
        flags: 0,
    };
    // SAFETY: `state` has the documented STICKYKEYS layout and its size is passed
    // exactly as required by SPI_GETSTICKYKEYS.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETSTICKYKEYS,
            state.cb_size,
            (&mut state as *mut StickyKeys).cast(),
            0,
        )
    };
    if ok == 0 {
        // SAFETY: GetLastError has no preconditions.
        bail!("SystemParametersInfoW failed: Windows error {}", unsafe {
            GetLastError()
        });
    }
    Ok((state.flags & (SKF_STICKYKEYSON | SKF_HOTKEYACTIVE))
        == (SKF_STICKYKEYSON | SKF_HOTKEYACTIVE))
}

#[cfg(not(windows))]
pub(crate) fn read_sticky_keys() -> Result<bool> {
    bail!("Sticky Keys readback is only available on Windows")
}

#[cfg(windows)]
pub(crate) fn replace_file_atomic(temp: &std::path::Path, target: &std::path::Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let temp_w: Vec<u16> = temp
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let target_w: Vec<u16> = target
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: Both paths are NUL-terminated UTF-16 buffers that remain alive for
    // the duration of the call; the flags request same-volume replacement.
    let ok = unsafe {
        MoveFileExW(
            temp_w.as_ptr(),
            target_w.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if ok == 0 {
        use windows_sys::Win32::Foundation::GetLastError;
        // SAFETY: GetLastError has no preconditions.
        bail!("MoveFileExW failed: Windows error {}", unsafe {
            GetLastError()
        });
    }
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn replace_file_atomic(temp: &std::path::Path, target: &std::path::Path) -> Result<()> {
    std::fs::rename(temp, target)?;
    Ok(())
}
