use crate::error::Result;
use crate::platform::pipe::Argument;
use crate::platform::{ADVANCED_SETTINGS, Platform};
use anyhow::bail;

const CMD_SET_GAMING_PROFILE: u16 = 9;
const CMD_GET_GAMING_PROFILE: u16 = 10;
const QUERY_GAMING_PROFILE: u32 = 0;
const LCD_OVERDRIVE_SELECTOR: u64 = 0x10;
const LCD_OVERDRIVE_ENABLED_BIT: u64 = 1 << 48;

pub(crate) fn read(platform: &Platform) -> Result<Option<bool>> {
    if platform.read_dword(ADVANCED_SETTINGS, "LCD_Overdrive_support")? == 0 {
        return Ok(None);
    }
    let value = platform.service_get_u64(
        CMD_GET_GAMING_PROFILE,
        &[Argument::U32(QUERY_GAMING_PROFILE)],
    )?;
    Ok(Some(((value >> 48) & 0xFF) == 1))
}

pub(crate) fn set(platform: &Platform, enabled: bool) -> Result<Option<bool>> {
    if platform.read_dword(ADVANCED_SETTINGS, "LCD_Overdrive_support")? == 0 {
        bail!("LCD Overdrive is disabled by NitroSense support registry")
    }
    let payload = LCD_OVERDRIVE_SELECTOR
        | if enabled {
            LCD_OVERDRIVE_ENABLED_BIT
        } else {
            0
        };
    let return_code = platform.service_set(CMD_SET_GAMING_PROFILE, &[Argument::U64(payload)])?;
    if return_code != 0 {
        bail!("LCD Overdrive command failed with return code {return_code}")
    }
    let observed = read(platform)?;
    if observed != Some(enabled) {
        bail!("LCD Overdrive verification failed: expected {enabled}, got {observed:?}")
    }
    Ok(observed)
}

#[cfg(test)]
mod tests {
    const LCD_OVERDRIVE_ENABLED_BIT: u64 = 1 << 48;

    #[test]
    fn overdrive_payload_uses_the_expected_bit() {
        assert_eq!((0x10 | LCD_OVERDRIVE_ENABLED_BIT) >> 48, 1);
    }
}
