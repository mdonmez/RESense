use crate::error::Result;
use crate::platform::pipe::{Argument, PipeClient};
use anyhow::Context;

pub const NITROSENSE: &str = r"SOFTWARE\OEM\NitroSense";
pub const FAN_CONTROL: &str = r"SOFTWARE\OEM\NitroSense\FanControl";
pub const OVERCLOCK: &str = r"SOFTWARE\OEM\NitroSense\Overclock";
pub const ADVANCED_SETTINGS: &str = r"SOFTWARE\OEM\NitroSense\AdvanceSettings";
pub const LIGHT_SETTING: &str = r"SOFTWARE\OEM\NitroSense\LightSetting";
pub const BIOS: &str = r"HARDWARE\DESCRIPTION\System\BIOS";

pub(crate) fn read_dword(path: &str, name: &str) -> Result<u32> {
    platform::read_dword(path, name).with_context(|| format!("reading HKLM\\{path}\\{name}"))
}

pub(crate) fn read_optional_string(path: &str, name: &str) -> Result<Option<String>> {
    platform::read_optional_string(path, name)
        .with_context(|| format!("reading HKLM\\{path}\\{name}"))
}

pub(crate) fn set_dwords(service: &PipeClient, updates: &[(&str, &str, u32)]) -> Result<()> {
    for (path, name, value) in updates {
        let full_path = format!(r"HKEY_LOCAL_MACHINE\{path}");
        let args = [
            Argument::Utf16(&full_path),
            Argument::Utf16(name),
            Argument::U32(4),
            Argument::U32(*value),
        ];
        let reply = service.set(3, &args)?;
        if reply.return_code != 0 {
            anyhow::bail!(
                "registry write failed for HKLM\\{path}\\{name}: return code {}",
                reply.return_code
            );
        }
    }
    Ok(())
}

#[cfg(windows)]
mod platform {
    use crate::error::Result;
    use std::io;
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    pub fn read_dword(path: &str, name: &str) -> Result<u32> {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path)?;
        Ok(key.get_value(name)?)
    }

    pub fn read_optional_string(path: &str, name: &str) -> Result<Option<String>> {
        let key = match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
            Ok(key) => key,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        match key.get_value(name) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::Result;
    use anyhow::bail;

    pub fn read_dword(_path: &str, _name: &str) -> Result<u32> {
        bail!("HKLM reads are only available on Windows")
    }

    pub fn read_optional_string(_path: &str, _name: &str) -> Result<Option<String>> {
        bail!("HKLM reads are only available on Windows")
    }
}
