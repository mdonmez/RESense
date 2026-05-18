use crate::cli::{SoundBackend, SoundPreset};
use crate::error::Result;
use crate::platform::{pipe, registry, session};
use anyhow::bail;
use serde::Serialize;
use std::{thread, time::Duration};

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
    let resolved = resolve_supported_backend(backend)?;
    let mode_code = preset_code(preset);
    send_admin_set(CMD_SET_DTS_SOUND_MODE, mode_code)?;
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
    read_backend(resolve_supported_backend(SoundBackend::Auto)?)
}

fn read_backend(backend: SoundBackend) -> Result<SoundState> {
    let mode_code = send_admin_get(CMD_GET_DTS_SOUND_MODE)?;
    let reliability = if mode_code == 9 {
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

fn resolve_supported_backend(backend: SoundBackend) -> Result<SoundBackend> {
    match backend {
        SoundBackend::Auto | SoundBackend::Dts if dts_supported() => Ok(SoundBackend::Dts),
        SoundBackend::Auto | SoundBackend::Dts => bail!(
            "the current default output is not on the validated DTS path; only the internal-speaker DTS sound path is supported right now"
        ),
    }
}

fn dts_supported() -> bool {
    registry::read_hklm_dword(registry::NITROSENSE, "DTS_Audio_Support").unwrap_or(0) == 1
}

fn send_admin_get(cmd_code: u16) -> Result<i32> {
    let mut last_error = None;
    for session_id in session::global_candidate_session_ids() {
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
    for session_id in session::global_candidate_session_ids() {
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
        SoundBackend::Auto => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_dts_presets() {
        assert_eq!(preset_code(SoundPreset::Shooter), 5);
        assert_eq!(preset_code(SoundPreset::Auto), 10);
    }
}
