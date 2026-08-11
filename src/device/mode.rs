use super::OperationMode;
use crate::error::Result;
use crate::platform::pipe::Argument;
use crate::platform::{OVERCLOCK, Platform};
use anyhow::bail;
use std::{thread, time::Duration};

const CMD_SET_OPERATION_MODE: u16 = 30;
const CMD_GET_GAMING_MISC_SETTING: u16 = 34;
const CMD_ADMIN_SET_WHISPERMODE: u16 = 15;
const OPERATION_MODE_QUERY: u32 = 11;

pub(crate) fn read(platform: &Platform) -> Result<OperationMode> {
    let value = platform.service_get_u64(
        CMD_GET_GAMING_MISC_SETTING,
        &[Argument::U32(OPERATION_MODE_QUERY)],
    )?;
    mode_from_code(((value >> 8) & 0xFF) as u8)
}

pub(crate) fn set(
    platform: &Platform,
    mode: OperationMode,
    skip_whispermode: bool,
) -> Result<OperationMode> {
    if !skip_whispermode {
        platform.shared_admin_fire(
            CMD_ADMIN_SET_WHISPERMODE,
            &[Argument::U32(matches!(mode, OperationMode::Quiet) as u32)],
        )?;
        thread::sleep(Duration::from_secs(1));
    }

    let expected_code = mode_code(mode);
    let _return_code = platform.service_set(
        CMD_SET_OPERATION_MODE,
        &[Argument::U32(expected_code as u32)],
    )?;

    let mut observed = None;
    for _ in 0..10 {
        let current = read(platform)?;
        if current == mode {
            observed = Some(current);
            break;
        }
        observed = Some(current);
        thread::sleep(Duration::from_millis(200));
    }
    if observed != Some(mode) {
        bail!("operation mode verification failed: expected {mode:?}, got {observed:?}");
    }
    platform.set_dwords(&[(OVERCLOCK, "CurrentOperationMode", expected_code as u32)])?;
    Ok(mode)
}

fn mode_code(mode: OperationMode) -> u8 {
    match mode {
        OperationMode::Quiet => 0,
        OperationMode::Default => 1,
        OperationMode::Performance => 4,
    }
}

fn mode_from_code(code: u8) -> Result<OperationMode> {
    match code {
        0 => Ok(OperationMode::Quiet),
        1 => Ok(OperationMode::Default),
        4 => Ok(OperationMode::Performance),
        _ => bail!("unsupported operation mode code {code}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_valid_mode_codes() {
        assert_eq!(mode_code(OperationMode::Quiet), 0);
        assert_eq!(mode_code(OperationMode::Default), 1);
        assert_eq!(mode_code(OperationMode::Performance), 4);
    }
}
