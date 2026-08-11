use super::SoundPreset;
use crate::error::Result;
use crate::platform::pipe::Argument;
use crate::platform::{NITROSENSE, Platform};
use anyhow::bail;
use std::{thread, time::Duration};

const CMD_GET_DTS_SOUND_MODE: u16 = 13;
const CMD_SET_DTS_SOUND_MODE: u16 = 14;

pub(crate) fn read(platform: &Platform) -> Result<Option<SoundPreset>> {
    if platform.read_dword(NITROSENSE, "DTS_Audio_Support")? != 1 {
        return Ok(None);
    }
    let code = platform.shared_admin_get_u32(CMD_GET_DTS_SOUND_MODE, &[])? as i32;
    preset_from_code(code)
}

pub(crate) fn set(platform: &Platform, preset: SoundPreset) -> Result<SoundPreset> {
    if platform.read_dword(NITROSENSE, "DTS_Audio_Support")? != 1 {
        bail!("the current default output is not on the validated DTS path")
    }
    platform.shared_admin_fire(
        CMD_SET_DTS_SOUND_MODE,
        &[Argument::U32(preset_code(preset))],
    )?;
    thread::sleep(Duration::from_millis(100));
    match read(platform)? {
        Some(observed) if observed == preset => Ok(observed),
        Some(observed) => {
            bail!("sound preset verification failed: expected {preset:?}, got {observed:?}")
        }
        None => bail!("sound preset became unavailable during verification"),
    }
}

fn preset_code(preset: SoundPreset) -> u32 {
    match preset {
        SoundPreset::Music => 0,
        SoundPreset::Movies => 1,
        SoundPreset::Voice => 2,
        SoundPreset::Strategy => 3,
        SoundPreset::Rpg => 4,
        SoundPreset::Shooter => 5,
        SoundPreset::Custom => 6,
        SoundPreset::Auto => 10,
    }
}

fn preset_from_code(code: i32) -> Result<Option<SoundPreset>> {
    match code {
        0 => Ok(Some(SoundPreset::Music)),
        1 => Ok(Some(SoundPreset::Movies)),
        2 => Ok(Some(SoundPreset::Voice)),
        3 => Ok(Some(SoundPreset::Strategy)),
        4 => Ok(Some(SoundPreset::Rpg)),
        5 => Ok(Some(SoundPreset::Shooter)),
        6 => Ok(Some(SoundPreset::Custom)),
        9 => Ok(None),
        10 => Ok(Some(SoundPreset::Auto)),
        _ => bail!("unsupported DTS sound preset code {code}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_sound_codes() {
        assert_eq!(preset_code(SoundPreset::Shooter), 5);
        assert_eq!(preset_code(SoundPreset::Auto), 10);
        assert_eq!(preset_from_code(9).unwrap(), None);
    }
}
