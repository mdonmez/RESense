use super::{FanChange, FanControl, FanMode, FanReading, FanState, Percent};
use crate::error::Result;
use crate::platform::pipe::Argument;
use crate::platform::{FAN_CONTROL, Platform};
use anyhow::bail;

const CMD_GET_GAMING_SYSINFO: u16 = 13;
const CMD_SET_FAN_GROUP_BEHAVIOR: u16 = 15;
const CMD_SET_FAN_GROUP_SPEED: u16 = 16;
const SYSINFO_CPU_TEMP: u32 = 1;
const SYSINFO_CPU_FAN_SPEED: u32 = 2;
const SYSINFO_GPU_FAN_SPEED: u32 = 6;
const SYSINFO_GPU_TEMP: u32 = 10;
const FAN_MODE_AUTO_ENCODED: u64 = 9 | 4_259_840;
const FAN_MODE_MAX_ENCODED: u64 = 9 | 8_519_680;
const CUSTOM_MODE_CODE: u32 = 2;
const AUTO_MODE_CODE: u32 = 0;
const MAX_MODE_CODE: u32 = 1;
const AUTO_BEHAVIOR_VALUE: u64 = 1;
const MANUAL_BEHAVIOR_VALUE: u64 = 3;
const CUSTOM_GPU_AUTO_FLAG: u64 = 0x40;
const CUSTOM_GPU_MANUAL_FLAG: u64 = 0xC0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Group {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct CustomState {
    cpu: FanControl,
    gpu: FanControl,
}

pub(crate) fn read(platform: &Platform) -> Result<FanState> {
    let custom = read_custom(platform)?;
    let mode_code = platform.read_dword(FAN_CONTROL, "CurrentFanMode")?;
    let mode = mode_from_code(mode_code)?;
    Ok(FanState {
        mode,
        cpu: FanReading {
            temperature_c: read_health(platform, SYSINFO_CPU_TEMP)?,
            rpm: read_health(platform, SYSINFO_CPU_FAN_SPEED)?,
            control: custom.cpu,
        },
        gpu: FanReading {
            temperature_c: read_health(platform, SYSINFO_GPU_TEMP)?,
            rpm: read_health(platform, SYSINFO_GPU_FAN_SPEED)?,
            control: custom.gpu,
        },
    })
}

pub(crate) fn set_mode(platform: &Platform, mode: FanMode) -> Result<FanState> {
    let payload = match mode {
        FanMode::Auto => FAN_MODE_AUTO_ENCODED,
        FanMode::Max => FAN_MODE_MAX_ENCODED,
        FanMode::Custom => bail!("custom mode requires per-fan changes"),
    };
    let return_code =
        platform.service_set(CMD_SET_FAN_GROUP_BEHAVIOR, &[Argument::U64(payload)])?;
    ensure_success(CMD_SET_FAN_GROUP_BEHAVIOR, return_code)?;
    platform.set_dwords(&[(FAN_CONTROL, "CurrentFanMode", mode_code(mode))])?;
    let state = read(platform)?;
    if state.mode != mode {
        bail!(
            "fan mode verification failed: expected {mode:?}, got {:?}",
            state.mode
        );
    }
    Ok(state)
}

pub(crate) fn set_custom(
    platform: &Platform,
    request: super::FanCustomRequest,
) -> Result<FanState> {
    let current = read_custom(platform)?;
    let merged = CustomState {
        cpu: request
            .cpu
            .map_or(current.cpu, |change| apply_change(current.cpu, change)),
        gpu: request
            .gpu
            .map_or(current.gpu, |change| apply_change(current.gpu, change)),
    };
    let return_code = platform.service_set(
        CMD_SET_FAN_GROUP_BEHAVIOR,
        &[Argument::U64(encode_behavior(merged))],
    )?;
    ensure_success(CMD_SET_FAN_GROUP_BEHAVIOR, return_code)?;

    if let Some(FanChange::Manual(percent)) = request.cpu {
        set_speed(platform, Group::Cpu, percent)?;
    }
    if let Some(FanChange::Manual(percent)) = request.gpu {
        set_speed(platform, Group::Gpu, percent)?;
    }

    sync_custom(platform, merged)?;
    let state = read(platform)?;
    if state.mode != FanMode::Custom
        || state.cpu.control != merged.cpu
        || state.gpu.control != merged.gpu
    {
        bail!("custom fan control verification failed");
    }
    Ok(state)
}

fn apply_change(previous: FanControl, change: FanChange) -> FanControl {
    match change {
        FanChange::Auto => FanControl::Auto {
            remembered_percent: previous.remembered_percent(),
        },
        FanChange::Manual(percent) => FanControl::Manual { percent },
    }
}

fn set_speed(platform: &Platform, group: Group, percent: Percent) -> Result<()> {
    let return_code = platform.service_set(
        CMD_SET_FAN_GROUP_SPEED,
        &[Argument::U64(encode_speed(group, percent))],
    )?;
    ensure_success(CMD_SET_FAN_GROUP_SPEED, return_code)
}

fn read_health(platform: &Platform, index: u32) -> Result<u16> {
    let query = 1 | (index << 8);
    let value = platform.service_get_u64(CMD_GET_GAMING_SYSINFO, &[Argument::U32(query)])?;
    let value = ((value >> 8) & 0xFFFF) as u16;
    Ok(value)
}

fn read_custom(platform: &Platform) -> Result<CustomState> {
    Ok(CustomState {
        cpu: read_control(platform, "CPUFanPercentage", "CPUFanCustomAuto")?,
        gpu: read_control(platform, "GPU1FanPercentage", "GPU1FanCustomAuto")?,
    })
}

fn read_control(platform: &Platform, percent_name: &str, auto_name: &str) -> Result<FanControl> {
    let percent = Percent::new(platform.read_dword(FAN_CONTROL, percent_name)? as u8)?;
    let auto = platform.read_dword(FAN_CONTROL, auto_name)? != 0;
    Ok(if auto {
        FanControl::Auto {
            remembered_percent: percent,
        }
    } else {
        FanControl::Manual { percent }
    })
}

fn sync_custom(platform: &Platform, state: CustomState) -> Result<()> {
    platform.set_dwords(&[
        (FAN_CONTROL, "CurrentFanMode", CUSTOM_MODE_CODE),
        (
            FAN_CONTROL,
            "CPUFanPercentage",
            state.cpu.remembered_percent().get() as u32,
        ),
        (
            FAN_CONTROL,
            "CPUFanCustomAuto",
            matches!(state.cpu, FanControl::Auto { .. }) as u32,
        ),
        (
            FAN_CONTROL,
            "GPU1FanPercentage",
            state.gpu.remembered_percent().get() as u32,
        ),
        (
            FAN_CONTROL,
            "GPU1FanCustomAuto",
            matches!(state.gpu, FanControl::Auto { .. }) as u32,
        ),
    ])
}

fn encode_speed(group: Group, percent: Percent) -> u64 {
    selector(group) | ((percent.get() as u64) << 8)
}

fn encode_behavior(state: CustomState) -> u64 {
    let cpu = if matches!(state.cpu, FanControl::Auto { .. }) {
        AUTO_BEHAVIOR_VALUE
    } else {
        MANUAL_BEHAVIOR_VALUE
    };
    let gpu = if matches!(state.gpu, FanControl::Auto { .. }) {
        CUSTOM_GPU_AUTO_FLAG
    } else {
        CUSTOM_GPU_MANUAL_FLAG
    };
    9 | ((gpu | cpu) << 16)
}

fn selector(group: Group) -> u64 {
    match group {
        Group::Cpu => 1,
        Group::Gpu => 4,
    }
}

fn mode_code(mode: FanMode) -> u32 {
    match mode {
        FanMode::Auto => AUTO_MODE_CODE,
        FanMode::Max => MAX_MODE_CODE,
        FanMode::Custom => CUSTOM_MODE_CODE,
    }
}

fn mode_from_code(code: u32) -> Result<FanMode> {
    match code {
        AUTO_MODE_CODE => Ok(FanMode::Auto),
        MAX_MODE_CODE => Ok(FanMode::Max),
        CUSTOM_MODE_CODE => Ok(FanMode::Custom),
        _ => bail!("unsupported fan mode code {code}"),
    }
}

fn ensure_success(command: u16, return_code: u32) -> Result<()> {
    if return_code != 0 {
        bail!("PredatorSense command {command} failed with return code {return_code}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_custom_speed() {
        assert_eq!(
            encode_speed(Group::Cpu, Percent::new(70).unwrap()),
            1 | (70 << 8)
        );
        assert_eq!(
            encode_speed(Group::Gpu, Percent::new(80).unwrap()),
            4 | (80 << 8)
        );
    }

    #[test]
    fn encodes_exact_custom_behavior() {
        let state = CustomState {
            cpu: FanControl::Manual {
                percent: Percent::new(50).unwrap(),
            },
            gpu: FanControl::Auto {
                remembered_percent: Percent::new(50).unwrap(),
            },
        };
        assert_eq!(encode_behavior(state), 0x430009);
    }

    #[test]
    fn automatic_changes_preserve_the_remembered_percent() {
        let previous = FanControl::Manual {
            percent: Percent::new(73).unwrap(),
        };
        assert_eq!(
            apply_change(previous, FanChange::Auto),
            FanControl::Auto {
                remembered_percent: Percent::new(73).unwrap()
            }
        );
    }
}
