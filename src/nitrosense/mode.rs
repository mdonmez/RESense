use crate::cli::OperatingMode;
use crate::error::Result;
use crate::platform::{pipe, registry, session};
use anyhow::bail;
use serde::Serialize;
use std::{thread, time::Duration};

const CMD_SET_OPERATION_MODE: u16 = 30;
const CMD_GET_GAMING_MISC_SETTING: u16 = 34;
const CMD_ADMIN_SET_WHISPERMODE: u16 = 15;
const OPERATION_MODE_QUERY: u32 = 11;

#[derive(Debug, Clone, Serialize)]
pub struct OperationModeState {
    pub mode: String,
    pub mode_code: u8,
    pub status: u8,
    pub source: String,
}

pub fn set_operation_mode(mode: OperatingMode, skip_whispermode: bool) -> Result<()> {
    let mode_code = mode_code(mode);
    if !skip_whispermode {
        let whispermode_enabled = matches!(mode, OperatingMode::Quiet);
        let _ = try_set_whispermode(whispermode_enabled);
        thread::sleep(Duration::from_secs(1));
    }

    let (raw, return_code) =
        pipe::service_set(CMD_SET_OPERATION_MODE, &[pipe::u32_arg(mode_code as u32)])?;
    let state = wait_for_mode(mode_code)?;
    if state.mode_code != mode_code {
        bail!(
            "operation mode verification failed: requested_mode_code={mode_code} return_code={return_code} reply={} live_mode_code={} live_status={}",
            hex(&raw),
            state.mode_code,
            state.status
        );
    }
    registry::set_hklm_dword(
        registry::OVERCLOCK,
        "CurrentOperationMode",
        mode_code as u32,
    )?;
    Ok(())
}

pub fn read_state() -> Result<OperationModeState> {
    let (_, value) = pipe::service_get_u64(
        CMD_GET_GAMING_MISC_SETTING,
        &[pipe::u32_arg(OPERATION_MODE_QUERY)],
    )?;
    let status = (value & 0xFF) as u8;
    let mode_code = ((value >> 8) & 0xFF) as u8;
    Ok(OperationModeState {
        mode: mode_name(mode_code).to_string(),
        mode_code,
        status,
        source: "service cmd 34/query 11".to_string(),
    })
}

fn try_set_whispermode(enabled: bool) -> Result<()> {
    let mut last_error = None;
    for session_id in session::global_candidate_session_ids() {
        let pipe_name = session::admin_pipe_name(session_id);
        match pipe::send_fire_and_forget(
            &pipe_name,
            CMD_ADMIN_SET_WHISPERMODE,
            &[pipe::u32_arg(enabled as u32)],
        ) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
    }
    if let Some(error) = last_error {
        Err(error)
    } else {
        bail!("no admin-agent session candidates")
    }
}

fn wait_for_mode(expected_mode_code: u8) -> Result<OperationModeState> {
    let mut last_state = None;
    for _ in 0..10 {
        let state = read_state()?;
        if state.mode_code == expected_mode_code {
            return Ok(state);
        }
        last_state = Some(state);
        thread::sleep(Duration::from_millis(200));
    }
    if let Some(state) = last_state {
        Ok(state)
    } else {
        read_state()
    }
}

fn mode_code(mode: OperatingMode) -> u8 {
    match mode {
        OperatingMode::Quiet => 0,
        OperatingMode::Default => 1,
        OperatingMode::Performance => 4,
    }
}

fn mode_name(code: u8) -> &'static str {
    match code {
        0 => "quiet",
        1 => "default",
        4 => "performance",
        _ => "unknown",
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_modes() {
        assert_eq!(mode_code(OperatingMode::Quiet), 0);
        assert_eq!(mode_code(OperatingMode::Default), 1);
        assert_eq!(mode_code(OperatingMode::Performance), 4);
    }
}
