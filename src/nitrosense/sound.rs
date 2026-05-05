use crate::cli::{SoundBackend, SoundPreset};
use crate::error::Result;
use crate::platform::{pipe, registry, session};
use anyhow::bail;
use serde::Serialize;
use std::{thread, time::Duration};

const CMD_GET_WAVES_SOUND_MODE: u16 = 11;
const CMD_SET_WAVES_SOUND_MODE: u16 = 12;
const CMD_GET_DTS_SOUND_MODE: u16 = 13;
const CMD_SET_DTS_SOUND_MODE: u16 = 14;
const ADMIN_GET_REPLY_SIZE: usize = 9;

#[derive(Debug, Clone, Serialize)]
pub struct SoundState {
    pub backend: String,
    pub dts_supported: bool,
    pub mode: String,
    pub mode_code: i32,
    pub reliability: String,
}

pub fn set_preset(backend: SoundBackend, preset: SoundPreset) -> Result<()> {
    let resolved = resolve_backend(backend);
    let mode_code = preset_code(resolved, preset)?;
    let set_cmd = match resolved {
        SoundBackend::Dts => CMD_SET_DTS_SOUND_MODE,
        SoundBackend::Waves => CMD_SET_WAVES_SOUND_MODE,
        SoundBackend::Auto => unreachable!("backend should be resolved"),
    };
    send_admin_set(set_cmd, mode_code)?;
    thread::sleep(Duration::from_millis(100));
    let state = read_backend(resolved)?;
    if state.mode_code != mode_code as i32 {
        bail!(
            "sound preset verification failed: expected {mode_code}, got {}",
            state.mode_code
        );
    }
    Ok(())
}

pub fn read_state() -> Result<SoundState> {
    read_backend(resolve_backend(SoundBackend::Auto))
}

fn read_backend(backend: SoundBackend) -> Result<SoundState> {
    let get_cmd = match backend {
        SoundBackend::Dts => CMD_GET_DTS_SOUND_MODE,
        SoundBackend::Waves => CMD_GET_WAVES_SOUND_MODE,
        SoundBackend::Auto => unreachable!("backend should be resolved"),
    };
    let mode_code = send_admin_get(get_cmd)?;
    let reliability = if backend == SoundBackend::Dts && mode_code == 9 {
        "unavailable"
    } else {
        "live"
    };
    Ok(SoundState {
        backend: backend.to_string(),
        dts_supported: dts_supported(),
        mode: preset_name(backend, mode_code).to_string(),
        mode_code,
        reliability: reliability.to_string(),
    })
}

fn resolve_backend(backend: SoundBackend) -> SoundBackend {
    match backend {
        SoundBackend::Auto if dts_supported() => SoundBackend::Dts,
        SoundBackend::Auto => SoundBackend::Waves,
        explicit => explicit,
    }
}

fn dts_supported() -> bool {
    registry::read_hklm_dword(registry::NITROSENSE, "DTS_Audio_Support").unwrap_or(0) == 1
}

fn send_admin_get(cmd_code: u16) -> Result<i32> {
    let mut last_error = None;
    for session_id in session::candidate_session_ids() {
        let pipe_name = session::admin_pipe_name(session_id);
        match pipe::send_set(&pipe_name, cmd_code, &[], ADMIN_GET_REPLY_SIZE) {
            Ok((raw, _)) => return Ok(i32::from_le_bytes(raw[5..9].try_into()?)),
            Err(error) => last_error = Some(error),
        }
    }
    match last_error {
        Some(error) => Err(error),
        None => bail!("no admin-agent session candidates"),
    }
}

fn send_admin_set(cmd_code: u16, mode_code: u32) -> Result<()> {
    let mut last_error = None;
    for session_id in session::candidate_session_ids() {
        let pipe_name = session::admin_pipe_name(session_id);
        match pipe::send_fire_and_forget(&pipe_name, cmd_code, &[pipe::u32_arg(mode_code)]) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
    }
    match last_error {
        Some(error) => Err(error),
        None => bail!("no admin-agent session candidates"),
    }
}

fn preset_code(backend: SoundBackend, preset: SoundPreset) -> Result<u32> {
    match backend {
        SoundBackend::Dts => match preset {
            SoundPreset::Music => Ok(0),
            SoundPreset::Movies => Ok(1),
            SoundPreset::Voice => Ok(2),
            SoundPreset::Strategy => Ok(3),
            SoundPreset::Rpg => Ok(4),
            SoundPreset::Shooter => Ok(5),
            SoundPreset::Custom => Ok(6),
            SoundPreset::Auto => Ok(10),
        },
        SoundBackend::Waves => match preset {
            SoundPreset::Music => Ok(0),
            SoundPreset::Movies => Ok(1),
            SoundPreset::Voice => Ok(3),
            _ => bail!("{preset} is not supported by the Waves backend"),
        },
        SoundBackend::Auto => unreachable!("backend should be resolved"),
    }
}

fn preset_name(backend: SoundBackend, code: i32) -> &'static str {
    match backend {
        SoundBackend::Dts => match code {
            0 => "music",
            1 => "movies",
            2 => "voice",
            3 => "strategy",
            4 => "rpg",
            5 => "shooter",
            6 => "custom",
            9 => "unavailable",
            10 => "auto",
            _ => "unknown",
        },
        SoundBackend::Waves => match code {
            0 => "music",
            1 => "movies",
            2 => "general",
            3 => "voice",
            4 => "fps",
            5 => "sports",
            _ => "unknown",
        },
        SoundBackend::Auto => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_dts_presets() {
        assert_eq!(
            preset_code(SoundBackend::Dts, SoundPreset::Shooter).unwrap(),
            5
        );
        assert_eq!(
            preset_code(SoundBackend::Dts, SoundPreset::Auto).unwrap(),
            10
        );
    }
}
