use crate::error::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StickyKeysState {
    pub enabled: bool,
    pub hotkey_active: bool,
    pub nitrosense_enabled: bool,
    pub flags: u32,
}

pub fn read() -> Result<StickyKeysState> {
    platform::read()
}

#[cfg(windows)]
mod platform {
    use super::StickyKeysState;
    use crate::error::Result;
    use anyhow::bail;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETSTICKYKEYS, SystemParametersInfoW};

    const SKF_STICKYKEYSON: u32 = 0x0000_0001;
    const SKF_HOTKEYACTIVE: u32 = 0x0000_0004;

    #[repr(C)]
    struct StickyKeysRaw {
        cb_size: u32,
        flags: u32,
    }

    pub fn read() -> Result<StickyKeysState> {
        let mut state = StickyKeysRaw {
            cb_size: std::mem::size_of::<StickyKeysRaw>() as u32,
            flags: 0,
        };
        let ok = unsafe {
            SystemParametersInfoW(
                SPI_GETSTICKYKEYS,
                state.cb_size,
                (&mut state as *mut StickyKeysRaw).cast(),
                0,
            )
        };
        if ok == 0 {
            bail!("SystemParametersInfoW failed: Windows error {}", unsafe {
                GetLastError()
            });
        }
        let enabled = (state.flags & SKF_STICKYKEYSON) == SKF_STICKYKEYSON;
        let hotkey_active = (state.flags & SKF_HOTKEYACTIVE) == SKF_HOTKEYACTIVE;
        Ok(StickyKeysState {
            enabled,
            hotkey_active,
            nitrosense_enabled: enabled && hotkey_active,
            flags: state.flags,
        })
    }
}

#[cfg(not(windows))]
mod platform {
    use super::StickyKeysState;
    use crate::error::Result;
    use anyhow::bail;

    pub fn read() -> Result<StickyKeysState> {
        bail!("Sticky Keys readback is only available on Windows");
    }
}
