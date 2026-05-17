use crate::cli::{FanMode, FanSpeedArgs};
use crate::error::Result;
use crate::platform::{pipe, registry};
use anyhow::bail;
use serde::Serialize;
use std::collections::BTreeMap;

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
const DEFAULT_PERCENT: u8 = 50;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FanGroup {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, Serialize)]
pub struct FanCustomState {
    pub percent: u8,
    pub auto: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FanSpeedResult {
    pub mode: String,
    pub custom: BTreeMap<FanGroup, FanCustomState>,
    pub registry_synced: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthValue {
    pub value: u16,
    pub trusted: bool,
    pub query: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FanState {
    pub cpu_temperature_c: HealthValue,
    pub gpu_temperature_c: HealthValue,
    pub cpu_fan_rpm: HealthValue,
    pub gpu_fan_rpm: HealthValue,
    pub persisted_mode: String,
    pub persisted_mode_code: u32,
    pub exact_mode_detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_custom: Option<BTreeMap<FanGroup, FanCustomState>>,
    pub remembered_custom: BTreeMap<FanGroup, FanCustomState>,
    pub note: String,
}

pub fn set_mode(mode: FanMode) -> Result<()> {
    let (encoded, mode_code) = match mode {
        FanMode::Auto => (FAN_MODE_AUTO_ENCODED, AUTO_MODE_CODE),
        FanMode::Max => (FAN_MODE_MAX_ENCODED, MAX_MODE_CODE),
    };
    let (raw, return_code) =
        pipe::service_set(CMD_SET_FAN_GROUP_BEHAVIOR, &[pipe::u64_arg(encoded)])?;
    ensure_success(CMD_SET_FAN_GROUP_BEHAVIOR, &raw, return_code)?;
    registry::set_hklm_dword(registry::FAN_CONTROL, "CurrentFanMode", mode_code)?;
    Ok(())
}

pub fn set_speed(args: &FanSpeedArgs) -> Result<FanSpeedResult> {
    let mut merged = read_custom_state().unwrap_or_else(|_| default_custom_state());
    let mut updates = BTreeMap::new();

    if args.cpu.is_some() || args.cpu_auto {
        let state = FanCustomState {
            percent: args.cpu.unwrap_or(DEFAULT_PERCENT),
            auto: args.cpu_auto,
        };
        merged.insert(FanGroup::Cpu, state.clone());
        updates.insert(FanGroup::Cpu, state);
    }
    if args.gpu.is_some() || args.gpu_auto {
        let state = FanCustomState {
            percent: args.gpu.unwrap_or(DEFAULT_PERCENT),
            auto: args.gpu_auto,
        };
        merged.insert(FanGroup::Gpu, state.clone());
        updates.insert(FanGroup::Gpu, state);
    }

    let encoded = encode_exact_custom_behavior(&merged);
    let (raw, return_code) =
        pipe::service_set(CMD_SET_FAN_GROUP_BEHAVIOR, &[pipe::u64_arg(encoded)])?;
    ensure_success(CMD_SET_FAN_GROUP_BEHAVIOR, &raw, return_code)?;

    for (group, state) in &updates {
        if !state.auto {
            let speed = encode_custom_speed(*group, state.percent);
            let (raw, return_code) =
                pipe::service_set(CMD_SET_FAN_GROUP_SPEED, &[pipe::u64_arg(speed)])?;
            ensure_success(CMD_SET_FAN_GROUP_SPEED, &raw, return_code)?;
        }
    }

    sync_custom_registry(&merged, CUSTOM_MODE_CODE)?;
    Ok(FanSpeedResult {
        mode: "custom".to_string(),
        custom: merged,
        registry_synced: true,
    })
}

pub fn read_state() -> Result<FanState> {
    let cpu_temperature_c = read_health(SYSINFO_CPU_TEMP)?;
    let gpu_temperature_c = read_health(SYSINFO_GPU_TEMP)?;
    let cpu_fan_rpm = read_health(SYSINFO_CPU_FAN_SPEED)?;
    let gpu_fan_rpm = read_health(SYSINFO_GPU_FAN_SPEED)?;
    let persisted_mode_code = registry::read_hklm_dword(registry::FAN_CONTROL, "CurrentFanMode")?;
    let persisted_mode = mode_name(persisted_mode_code).to_string();
    let remembered_custom = read_custom_state()?;
    let exact_mode_detail = describe_exact_mode(&persisted_mode, &remembered_custom);
    let active_custom = if persisted_mode == "custom" {
        Some(remembered_custom.clone())
    } else {
        None
    };

    Ok(FanState {
        cpu_temperature_c,
        gpu_temperature_c,
        cpu_fan_rpm,
        gpu_fan_rpm,
        persisted_mode,
        persisted_mode_code,
        exact_mode_detail,
        active_custom,
        remembered_custom,
        note: "RPM and temperatures come from live service getters. Exact active fan mode is resolved from HKLM NitroSense FanControl: CurrentFanMode is the authoritative high-level mode, and the per-fan custom fields only describe the active state when CurrentFanMode=custom. This model was re-verified against NitroSense on 2026-05-17.".to_string(),
    })
}

fn read_health(index: u32) -> Result<HealthValue> {
    let query = 1 | (index << 8);
    let (_, value) = pipe::service_get_u64(CMD_GET_GAMING_SYSINFO, &[pipe::u32_arg(query)])?;
    Ok(HealthValue {
        value: ((value >> 8) & 0xFFFF) as u16,
        trusted: (value & 0xFF) == 0,
        query,
    })
}

fn read_custom_state() -> Result<BTreeMap<FanGroup, FanCustomState>> {
    let mut state = BTreeMap::new();
    state.insert(
        FanGroup::Cpu,
        FanCustomState {
            percent: registry::read_hklm_dword(registry::FAN_CONTROL, "CPUFanPercentage")? as u8,
            auto: registry::read_hklm_dword(registry::FAN_CONTROL, "CPUFanCustomAuto")? != 0,
        },
    );
    state.insert(
        FanGroup::Gpu,
        FanCustomState {
            percent: registry::read_hklm_dword(registry::FAN_CONTROL, "GPU1FanPercentage")? as u8,
            auto: registry::read_hklm_dword(registry::FAN_CONTROL, "GPU1FanCustomAuto")? != 0,
        },
    );
    Ok(state)
}

fn default_custom_state() -> BTreeMap<FanGroup, FanCustomState> {
    BTreeMap::from([
        (
            FanGroup::Cpu,
            FanCustomState {
                percent: DEFAULT_PERCENT,
                auto: true,
            },
        ),
        (
            FanGroup::Gpu,
            FanCustomState {
                percent: DEFAULT_PERCENT,
                auto: true,
            },
        ),
    ])
}

fn sync_custom_registry(state: &BTreeMap<FanGroup, FanCustomState>, mode_code: u32) -> Result<()> {
    registry::set_hklm_dwords(&[
        (registry::FAN_CONTROL, "CurrentFanMode", mode_code),
        (
            registry::FAN_CONTROL,
            "CPUFanPercentage",
            state[&FanGroup::Cpu].percent as u32,
        ),
        (
            registry::FAN_CONTROL,
            "CPUFanCustomAuto",
            state[&FanGroup::Cpu].auto as u32,
        ),
        (
            registry::FAN_CONTROL,
            "GPU1FanPercentage",
            state[&FanGroup::Gpu].percent as u32,
        ),
        (
            registry::FAN_CONTROL,
            "GPU1FanCustomAuto",
            state[&FanGroup::Gpu].auto as u32,
        ),
    ])
}

pub fn encode_custom_speed(group: FanGroup, percent: u8) -> u64 {
    selector(group) | ((percent as u64) << 8)
}

pub fn encode_exact_custom_behavior(state: &BTreeMap<FanGroup, FanCustomState>) -> u64 {
    let cpu = if state[&FanGroup::Cpu].auto {
        AUTO_BEHAVIOR_VALUE
    } else {
        MANUAL_BEHAVIOR_VALUE
    };
    let gpu = if state[&FanGroup::Gpu].auto {
        CUSTOM_GPU_AUTO_FLAG
    } else {
        CUSTOM_GPU_MANUAL_FLAG
    };
    9 | ((gpu | cpu) << 16)
}

fn selector(group: FanGroup) -> u64 {
    match group {
        FanGroup::Cpu => 1,
        FanGroup::Gpu => 4,
    }
}

fn mode_name(code: u32) -> &'static str {
    match code {
        AUTO_MODE_CODE => "auto",
        MAX_MODE_CODE => "max",
        CUSTOM_MODE_CODE => "custom",
        _ => "unknown",
    }
}

fn describe_exact_mode(mode: &str, custom: &BTreeMap<FanGroup, FanCustomState>) -> String {
    match mode {
        "auto" => "global_auto".to_string(),
        "max" => "max".to_string(),
        "custom" if custom[&FanGroup::Cpu].auto && custom[&FanGroup::Gpu].auto => {
            "custom_all_auto".to_string()
        }
        "custom" if custom[&FanGroup::Cpu].auto => "custom_gpu_manual".to_string(),
        "custom" if custom[&FanGroup::Gpu].auto => "custom_cpu_manual".to_string(),
        "custom" => "custom_cpu_gpu_manual".to_string(),
        _ => "unknown".to_string(),
    }
}

fn ensure_success(cmd: u16, raw: &[u8], return_code: u32) -> Result<()> {
    if return_code != 0 {
        bail!(
            "PredatorSense command {cmd} failed with return_code={return_code} reply={}",
            hex(raw)
        );
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_custom_speed() {
        assert_eq!(encode_custom_speed(FanGroup::Cpu, 70), 1 | (70 << 8));
        assert_eq!(encode_custom_speed(FanGroup::Gpu, 80), 4 | (80 << 8));
    }

    #[test]
    fn encodes_exact_custom_behavior() {
        let state = BTreeMap::from([
            (
                FanGroup::Cpu,
                FanCustomState {
                    percent: 50,
                    auto: false,
                },
            ),
            (
                FanGroup::Gpu,
                FanCustomState {
                    percent: 50,
                    auto: true,
                },
            ),
        ]);
        assert_eq!(encode_exact_custom_behavior(&state), 0x430009);
    }

    #[test]
    fn encodes_custom_all_auto_behavior() {
        let state = BTreeMap::from([
            (
                FanGroup::Cpu,
                FanCustomState {
                    percent: 50,
                    auto: true,
                },
            ),
            (
                FanGroup::Gpu,
                FanCustomState {
                    percent: 50,
                    auto: true,
                },
            ),
        ]);
        assert_eq!(encode_exact_custom_behavior(&state), 0x410009);
    }

    #[test]
    fn describes_exact_mode_variants() {
        let custom_all_auto = BTreeMap::from([
            (
                FanGroup::Cpu,
                FanCustomState {
                    percent: 50,
                    auto: true,
                },
            ),
            (
                FanGroup::Gpu,
                FanCustomState {
                    percent: 50,
                    auto: true,
                },
            ),
        ]);
        let custom_cpu_manual = BTreeMap::from([
            (
                FanGroup::Cpu,
                FanCustomState {
                    percent: 70,
                    auto: false,
                },
            ),
            (
                FanGroup::Gpu,
                FanCustomState {
                    percent: 50,
                    auto: true,
                },
            ),
        ]);
        let custom_gpu_manual = BTreeMap::from([
            (
                FanGroup::Cpu,
                FanCustomState {
                    percent: 70,
                    auto: true,
                },
            ),
            (
                FanGroup::Gpu,
                FanCustomState {
                    percent: 70,
                    auto: false,
                },
            ),
        ]);

        assert_eq!(describe_exact_mode("custom", &custom_all_auto), "custom_all_auto");
        assert_eq!(describe_exact_mode("custom", &custom_cpu_manual), "custom_cpu_manual");
        assert_eq!(describe_exact_mode("custom", &custom_gpu_manual), "custom_gpu_manual");
    }
}
