mod display;
mod fan;
mod keyboard;
mod mode;
mod sound;

use crate::error::Result;
use crate::platform::Platform;
use anyhow::bail;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Percent(u8);

impl Percent {
    pub fn new(value: u8) -> Result<Self> {
        if value > 100 {
            bail!("percentage must be between 0 and 100");
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Brightness(u8);

impl Brightness {
    pub fn new(value: u8) -> Result<Self> {
        if !(1..=5).contains(&value) {
            bail!("brightness must be between 1 and 5");
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct DynamicSpeed(u8);

impl DynamicSpeed {
    pub fn new(value: u8) -> Result<Self> {
        if !(1..=9).contains(&value) {
            bail!("dynamic speed must be between 1 and 9");
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub fn parse(value: &str) -> Result<Self> {
        let value = value.trim().strip_prefix('#').unwrap_or(value.trim());
        if value.len() != 6 || !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
            bail!("expected a six-digit RGB color, got {value}");
        }
        Ok(Self {
            red: u8::from_str_radix(&value[0..2], 16)?,
            green: u8::from_str_radix(&value[2..4], 16)?,
            blue: u8::from_str_radix(&value[4..6], 16)?,
        })
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{:02X}{:02X}{:02X}",
            self.red, self.green, self.blue
        )
    }
}

impl Serialize for Rgb {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FanMode {
    Auto,
    Max,
    Custom,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FanControl {
    Auto { remembered_percent: Percent },
    Manual { percent: Percent },
}

impl FanControl {
    pub fn mode_name(self) -> &'static str {
        match self {
            Self::Auto { .. } => "auto",
            Self::Manual { .. } => "manual",
        }
    }

    pub fn percent(self) -> Option<Percent> {
        match self {
            Self::Auto { .. } => None,
            Self::Manual { percent } => Some(percent),
        }
    }

    pub fn remembered_percent(self) -> Percent {
        match self {
            Self::Auto { remembered_percent } => remembered_percent,
            Self::Manual { percent } => percent,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FanReading {
    pub temperature_c: u16,
    pub rpm: u16,
    pub control: FanControl,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FanState {
    pub mode: FanMode,
    pub cpu: FanReading,
    pub gpu: FanReading,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FanChange {
    Auto,
    Manual(Percent),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FanCustomRequest {
    cpu: Option<FanChange>,
    gpu: Option<FanChange>,
}

impl FanCustomRequest {
    pub fn new(cpu: Option<FanChange>, gpu: Option<FanChange>) -> Result<Self> {
        if cpu.is_none() && gpu.is_none() {
            bail!("provide at least one fan change");
        }
        Ok(Self { cpu, gpu })
    }

    pub const fn cpu(self) -> Option<FanChange> {
        self.cpu
    }

    pub const fn gpu(self) -> Option<FanChange> {
        self.gpu
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationMode {
    Quiet,
    Default,
    Performance,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundPreset {
    Music,
    Movies,
    Voice,
    Strategy,
    Rpg,
    Shooter,
    Custom,
    Auto,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    FromLeft,
    FromRight,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicMode {
    Breathing,
    Neon,
    Shifting,
    Wave,
    Zoom,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ZoneChange {
    Off,
    Color(Rgb),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StaticRequest {
    zones: [Option<ZoneChange>; 4],
}

impl StaticRequest {
    pub fn new(zones: [Option<ZoneChange>; 4]) -> Result<Self> {
        if zones.iter().all(Option::is_none) {
            bail!("provide at least one zone change");
        }
        Ok(Self { zones })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DynamicRequest {
    mode: DynamicMode,
    speed: Option<DynamicSpeed>,
    color: Option<Rgb>,
    direction: Option<Direction>,
}

impl DynamicRequest {
    pub fn new(
        mode: DynamicMode,
        speed: Option<DynamicSpeed>,
        color: Option<Rgb>,
        direction: Option<Direction>,
    ) -> Result<Self> {
        let uses_color = !matches!(mode, DynamicMode::Neon);
        let uses_direction = matches!(mode, DynamicMode::Wave | DynamicMode::Shifting);
        if !uses_color && color.is_some() {
            bail!("{mode:?} does not use a color");
        }
        if !uses_direction && direction.is_some() {
            bail!("{mode:?} does not use a direction");
        }
        Ok(Self {
            mode,
            speed,
            color,
            direction,
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Zone {
    pub enabled: bool,
    pub color: Rgb,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DynamicEffect {
    Breathing { color: Rgb },
    Neon,
    Shifting { color: Rgb, direction: Direction },
    Wave { color: Rgb, direction: Direction },
    Zoom { color: Rgb },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DynamicLighting {
    pub effect: DynamicEffect,
    pub speed: DynamicSpeed,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LightingState {
    Static {
        zones: [Zone; 4],
    },
    Dynamic {
        zones: [Zone; 4],
        effect: DynamicLighting,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct KeyboardState {
    pub brightness: Brightness,
    pub lighting: LightingState,
    pub backlight_timeout: bool,
    pub sticky_keys: bool,
    pub win_menu_lock: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SystemState {
    pub fan: FanState,
    pub keyboard: KeyboardState,
    pub mode: OperationMode,
    pub display_overdrive: Option<bool>,
    pub sound: Option<SoundPreset>,
}

pub struct Device {
    platform: Platform,
}

impl Device {
    pub fn connect(allow_any_model: bool) -> Result<Self> {
        Ok(Self {
            platform: Platform::connect(allow_any_model)?,
        })
    }

    pub fn status(&self) -> Result<SystemState> {
        Ok(SystemState {
            fan: fan::read(&self.platform)?,
            keyboard: keyboard::read(&self.platform)?,
            mode: mode::read(&self.platform)?,
            display_overdrive: display::read(&self.platform)?,
            sound: sound::read(&self.platform)?,
        })
    }

    pub fn fan(&self) -> Result<FanState> {
        fan::read(&self.platform)
    }

    pub fn set_fan_auto(&self) -> Result<FanState> {
        self.ensure_fan_allowed()?;
        fan::set_mode(&self.platform, FanMode::Auto)
    }

    pub fn set_fan_max(&self) -> Result<FanState> {
        self.ensure_fan_allowed()?;
        fan::set_mode(&self.platform, FanMode::Max)
    }

    pub fn set_fan_custom(&self, request: FanCustomRequest) -> Result<FanState> {
        self.ensure_fan_allowed()?;
        fan::set_custom(&self.platform, request)
    }

    pub fn keyboard(&self) -> Result<KeyboardState> {
        keyboard::read(&self.platform)
    }

    pub fn set_keyboard_brightness(&self, brightness: Brightness) -> Result<KeyboardState> {
        keyboard::set_brightness(&self.platform, brightness)
    }

    pub fn set_keyboard_timeout(&self, enabled: bool) -> Result<KeyboardState> {
        keyboard::set_timeout(&self.platform, enabled)
    }

    pub fn set_keyboard_static(&self, request: StaticRequest) -> Result<KeyboardState> {
        keyboard::set_static(&self.platform, request)
    }

    pub fn set_keyboard_dynamic(&self, request: DynamicRequest) -> Result<KeyboardState> {
        keyboard::set_dynamic(&self.platform, request)
    }

    pub fn set_sticky_keys(&self, enabled: bool) -> Result<KeyboardState> {
        keyboard::set_sticky_keys(&self.platform, enabled)
    }

    pub fn set_win_menu_lock(&self, enabled: bool) -> Result<KeyboardState> {
        keyboard::set_win_menu_lock(&self.platform, enabled)
    }

    pub fn mode(&self) -> Result<OperationMode> {
        mode::read(&self.platform)
    }

    pub fn set_mode(&self, mode: OperationMode, skip_whispermode: bool) -> Result<OperationMode> {
        if matches!(mode, OperationMode::Quiet) {
            fan::set_mode(&self.platform, FanMode::Auto)?;
        }
        let result = mode::set(&self.platform, mode, skip_whispermode);
        if result.is_ok() && matches!(mode, OperationMode::Quiet) {
            fan::set_mode(&self.platform, FanMode::Auto)?;
        }
        result
    }

    pub fn display_overdrive(&self) -> Result<Option<bool>> {
        display::read(&self.platform)
    }

    pub fn set_display_overdrive(&self, enabled: bool) -> Result<Option<bool>> {
        display::set(&self.platform, enabled)
    }

    pub fn sound(&self) -> Result<Option<SoundPreset>> {
        sound::read(&self.platform)
    }

    pub fn set_sound(&self, preset: SoundPreset) -> Result<SoundPreset> {
        sound::set(&self.platform, preset)
    }

    fn ensure_fan_allowed(&self) -> Result<()> {
        if matches!(mode::read(&self.platform)?, OperationMode::Quiet) {
            bail!("RESense disables fan control while quiet mode is active")
        }
        Ok(())
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Device").finish_non_exhaustive()
    }
}
