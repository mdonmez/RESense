use crate::error::Result;
use crate::platform::pipe;
use anyhow::Context;

pub const NITROSENSE: &str = r"SOFTWARE\OEM\NitroSense";
pub const FAN_CONTROL: &str = r"SOFTWARE\OEM\NitroSense\FanControl";
pub const OVERCLOCK: &str = r"SOFTWARE\OEM\NitroSense\Overclock";
pub const ADVANCED_SETTINGS: &str = r"SOFTWARE\OEM\NitroSense\AdvanceSettings";
pub const LIGHT_SETTING: &str = r"SOFTWARE\OEM\NitroSense\LightSetting";

pub fn read_hklm_dword(path: &str, name: &str) -> Result<u32> {
    platform::read_hklm_dword(path, name).with_context(|| format!("reading HKLM\\{path}\\{name}"))
}

pub fn read_hklm_string(path: &str, name: &str) -> Result<String> {
    platform::read_hklm_string(path, name).with_context(|| format!("reading HKLM\\{path}\\{name}"))
}

pub fn read_hklm_dword_default(path: &str, name: &str, default: u32) -> u32 {
    read_hklm_dword(path, name).unwrap_or(default)
}

pub fn set_hklm_dword(path: &str, name: &str, value: u32) -> Result<()> {
    set_hklm_dwords(&[(path, name, value)])
}

pub fn set_hklm_dwords(updates: &[(&str, &str, u32)]) -> Result<()> {
    for (path, name, value) in updates {
        let full_path = format!(r"HKEY_LOCAL_MACHINE\{path}");
        let args = vec![
            pipe::utf16_string_arg(&full_path),
            pipe::utf16_string_arg(name),
            pipe::u32_arg(4),
            pipe::u32_arg(*value),
        ];
        pipe::send_fire_and_forget(pipe::SERVICE_PIPE_NAME, 3, &args)?;
    }
    Ok(())
}

#[cfg(windows)]
mod platform {
    use crate::error::Result;
    use anyhow::bail;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_DWORD, REG_SZ, RegCloseKey, RegOpenKeyExW,
        RegQueryValueExW,
    };

    pub fn read_hklm_dword(path: &str, name: &str) -> Result<u32> {
        let key = HklmKey::open(path)?;
        let name_w = wide(name);
        let mut value_type = 0u32;
        let mut data = 0u32;
        let mut data_len = std::mem::size_of::<u32>() as u32;
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                name_w.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                (&mut data as *mut u32).cast(),
                &mut data_len,
            )
        };

        if status != ERROR_SUCCESS {
            bail!("RegQueryValueExW failed with status {status}");
        }
        if value_type != REG_DWORD {
            bail!("registry value is not REG_DWORD");
        }
        Ok(data)
    }

    pub fn read_hklm_string(path: &str, name: &str) -> Result<String> {
        let key = HklmKey::open(path)?;
        let name_w = wide(name);
        let mut value_type = 0u32;
        let mut data_len = 0u32;
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                name_w.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                std::ptr::null_mut(),
                &mut data_len,
            )
        };
        if status != ERROR_SUCCESS {
            bail!("RegQueryValueExW size probe failed with status {status}");
        }
        if value_type != REG_SZ {
            bail!("registry value is not REG_SZ");
        }

        let mut data = vec![0u16; (data_len as usize).div_ceil(2)];
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                name_w.as_ptr(),
                std::ptr::null_mut(),
                &mut value_type,
                data.as_mut_ptr().cast(),
                &mut data_len,
            )
        };
        if status != ERROR_SUCCESS {
            bail!("RegQueryValueExW failed with status {status}");
        }
        while data.last() == Some(&0) {
            data.pop();
        }
        Ok(String::from_utf16_lossy(&data))
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    struct HklmKey(HKEY);

    impl HklmKey {
        fn open(path: &str) -> Result<Self> {
            let path_w = wide(path);
            let mut key: HKEY = std::ptr::null_mut();
            let status = unsafe {
                RegOpenKeyExW(HKEY_LOCAL_MACHINE, path_w.as_ptr(), 0, KEY_READ, &mut key)
            };
            if status != ERROR_SUCCESS {
                bail!("RegOpenKeyExW failed with status {status}");
            }
            Ok(Self(key))
        }
    }

    impl Drop for HklmKey {
        fn drop(&mut self) {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::Result;
    use anyhow::bail;

    pub fn read_hklm_dword(_path: &str, _name: &str) -> Result<u32> {
        bail!("HKLM reads are only available on Windows");
    }

    pub fn read_hklm_string(_path: &str, _name: &str) -> Result<String> {
        bail!("HKLM reads are only available on Windows");
    }
}
