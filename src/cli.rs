use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::fmt;

#[derive(Parser, Debug)]
#[command(name = "resense")]
#[command(about = "RESense command-line interface")]
pub struct Cli {
    #[arg(
        long = "dangerously-allow-any-model",
        global = true,
        help = "Bypass the AN515-58 model check and run on any machine"
    )]
    pub dangerously_allow_any_model: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Read current state")]
    Status(StatusArgs),
    #[command(about = "Manage fan control")]
    Fan(FanCommand),
    #[command(about = "Manage keyboard settings")]
    Keyboard(KeyboardCommand),
    #[command(about = "Set the operation mode")]
    Mode(ModeArgs),
    #[command(about = "Manage display settings")]
    Display(DisplayCommand),
    #[command(about = "Set the sound preset")]
    Sound(SoundArgs),
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    #[arg(value_enum, help = "Optional subsystem to read")]
    pub target: Option<StatusTarget>,
    #[arg(long, help = "Print JSON instead of human-readable text")]
    pub json: bool,
    #[arg(long, help = "Continue reading state at the selected interval")]
    pub watch: bool,
    #[arg(
        long,
        requires = "watch",
        value_parser = clap::value_parser!(u64),
        help = "Polling interval in seconds (default: 2)"
    )]
    pub interval: Option<u64>,
}

#[derive(Args, Debug)]
#[command(about = "Manage fan control")]
pub struct FanCommand {
    #[command(subcommand)]
    pub command: FanCommands,
}

#[derive(Subcommand, Debug)]
pub enum FanCommands {
    #[command(about = "Set both fans to automatic control")]
    Auto,
    #[command(about = "Set both fans to maximum speed")]
    Max,
    #[command(about = "Set per-fan manual or automatic control")]
    Custom(FanCustomArgs),
}

#[derive(Args, Debug)]
pub struct FanCustomArgs {
    #[arg(long, help = "CPU fan speed percentage")]
    pub cpu: Option<u8>,
    #[arg(long, help = "GPU fan speed percentage")]
    pub gpu: Option<u8>,
    #[arg(long = "cpu-auto", help = "Set the CPU fan to automatic mode")]
    pub cpu_auto: bool,
    #[arg(long = "gpu-auto", help = "Set the GPU fan to automatic mode")]
    pub gpu_auto: bool,
}

#[derive(Args, Debug)]
#[command(about = "Manage keyboard settings")]
pub struct KeyboardCommand {
    #[command(subcommand)]
    pub command: KeyboardCommands,
}

#[derive(Subcommand, Debug)]
pub enum KeyboardCommands {
    #[command(about = "Set keyboard brightness")]
    Brightness(KeyboardBrightnessArgs),
    #[command(about = "Enable or disable keyboard backlight timeout")]
    Timeout(ToggleArgs),
    #[command(about = "Set 4-zone static keyboard lighting")]
    Static(KeyboardStaticArgs),
    #[command(about = "Set a dynamic keyboard lighting effect")]
    Dynamic(KeyboardDynamicArgs),
    #[command(about = "Enable or disable Sticky Keys")]
    Sticky(ToggleArgs),
    #[command(about = "Enable or disable Windows/Menu key lock")]
    WinMenu(ToggleArgs),
}

#[derive(Args, Debug)]
pub struct KeyboardBrightnessArgs {
    #[arg(help = "Brightness level from 1 to 5")]
    pub level: u8,
}

#[derive(Args, Debug)]
pub struct KeyboardStaticArgs {
    #[arg(long, help = "Zone 1 color as a 6-digit hex value or off")]
    pub zone1: Option<String>,
    #[arg(long, help = "Zone 2 color as a 6-digit hex value or off")]
    pub zone2: Option<String>,
    #[arg(long, help = "Zone 3 color as a 6-digit hex value or off")]
    pub zone3: Option<String>,
    #[arg(long, help = "Zone 4 color as a 6-digit hex value or off")]
    pub zone4: Option<String>,
}

#[derive(Args, Debug)]
pub struct KeyboardDynamicArgs {
    #[arg(help = "Dynamic effect mode")]
    pub mode: KeyboardDynamicMode,
    #[arg(long, help = "Effect speed from 1 to 9")]
    pub speed: Option<u8>,
    #[arg(long, help = "Effect color as a 6-digit hex value")]
    pub color: Option<String>,
    #[arg(long, help = "Effect direction: fromleft or fromright")]
    pub direction: Option<Direction>,
}

#[derive(Args, Debug)]
pub struct ModeArgs {
    #[arg(help = "Operation mode")]
    pub mode: OperatingMode,
    #[arg(long = "skip-whispermode", help = "Skip WhisperMode integration")]
    pub skip_whispermode: bool,
}

#[derive(Args, Debug)]
#[command(about = "Manage display settings")]
pub struct DisplayCommand {
    #[command(subcommand)]
    pub command: DisplayCommands,
}

#[derive(Subcommand, Debug)]
pub enum DisplayCommands {
    #[command(about = "Enable or disable LCD overdrive")]
    Overdrive(ToggleArgs),
}

#[derive(Args, Debug)]
pub struct SoundArgs {
    #[arg(help = "Sound preset")]
    pub preset: SoundPreset,
}

#[derive(Args, Debug)]
pub struct ToggleArgs {
    #[arg(help = "Enable or disable the setting")]
    pub state: ToggleState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum FanMode {
    Auto,
    Max,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StatusTarget {
    Fan,
    Keyboard,
    Mode,
    Display,
    Sound,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum KeyboardDynamicMode {
    Breathing,
    Neon,
    Shifting,
    Wave,
    Zoom,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    FromLeft,
    FromRight,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    Quiet,
    Default,
    Performance,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ToggleState {
    Enable,
    Disable,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, ValueEnum)]
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

impl ToggleState {
    pub fn enabled(self) -> bool {
        matches!(self, Self::Enable)
    }
}

macro_rules! display_value {
    ($type:ty) => {
        impl fmt::Display for $type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", format!("{self:?}").to_ascii_lowercase())
            }
        }
    };
}

display_value!(FanMode);
display_value!(KeyboardDynamicMode);
display_value!(Direction);
display_value!(OperatingMode);
display_value!(ToggleState);
display_value!(SoundPreset);
display_value!(StatusTarget);

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_targeted_status() {
        let cli = Cli::try_parse_from(["resense", "status", "fan", "--json"]).unwrap();
        match cli.command {
            Commands::Status(args) => {
                assert_eq!(args.target, Some(StatusTarget::Fan));
                assert!(args.json);
                assert!(!args.watch);
            }
            _ => panic!("expected status command"),
        }
    }

    #[test]
    fn parses_direct_fan_commands() {
        let cli =
            Cli::try_parse_from(["resense", "fan", "custom", "--cpu", "70", "--gpu-auto"]).unwrap();

        match cli.command {
            Commands::Fan(command) => match command.command {
                FanCommands::Custom(args) => {
                    assert_eq!(args.cpu, Some(70));
                    assert!(args.gpu_auto);
                }
                _ => panic!("expected custom fan command"),
            },
            _ => panic!("expected fan command"),
        }
    }

    #[test]
    fn rejects_removed_command_forms() {
        for command in [
            vec!["resense", "get", "fan"],
            vec!["resense", "fan", "speed", "--cpu", "70"],
            vec!["resense", "keyboard", "backlight-timeout", "enable"],
            vec!["resense", "sound", "--backend", "dts", "music"],
        ] {
            assert!(
                Cli::try_parse_from(command.clone()).is_err(),
                "accepted {command:?}"
            );
        }
    }
}
