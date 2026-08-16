use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Parser, Debug)]
#[command(name = "resense", version)]
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
    #[command(about = "Enable or disable overdrive")]
    Overdrive(ToggleArgs),
    #[command(about = "Set the sound preset")]
    Sound(SoundArgs),
    #[command(about = "Check for and install updates")]
    Update,
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
    #[arg(
        long,
        conflicts_with = "cpu_auto",
        required_unless_present = "cpu_auto",
        help = "CPU fan speed percentage"
    )]
    pub cpu: Option<u8>,
    #[arg(
        long,
        conflicts_with = "gpu_auto",
        required_unless_present = "gpu_auto",
        help = "GPU fan speed percentage"
    )]
    pub gpu: Option<u8>,
    #[arg(
        long = "cpu-auto",
        conflicts_with = "cpu",
        required_unless_present = "cpu",
        help = "Set the CPU fan to automatic mode"
    )]
    pub cpu_auto: bool,
    #[arg(
        long = "gpu-auto",
        conflicts_with = "gpu",
        required_unless_present = "gpu",
        help = "Set the GPU fan to automatic mode"
    )]
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
    Dynamic(KeyboardDynamicCommand),
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
    #[arg(
        long,
        required = true,
        help = "Zone 1 color as a 6-digit hex value or off"
    )]
    pub zone1: Option<String>,
    #[arg(
        long,
        required = true,
        help = "Zone 2 color as a 6-digit hex value or off"
    )]
    pub zone2: Option<String>,
    #[arg(
        long,
        required = true,
        help = "Zone 3 color as a 6-digit hex value or off"
    )]
    pub zone3: Option<String>,
    #[arg(
        long,
        required = true,
        help = "Zone 4 color as a 6-digit hex value or off"
    )]
    pub zone4: Option<String>,
}

#[derive(Args, Debug)]
pub struct KeyboardDynamicCommand {
    #[command(subcommand)]
    pub command: KeyboardDynamicCommands,
}

#[derive(Subcommand, Debug)]
pub enum KeyboardDynamicCommands {
    #[command(about = "Breathing effect with color and speed")]
    Breathing(DynamicColorArgs),
    #[command(about = "Multicolor neon effect with speed")]
    Neon(DynamicSpeedArgs),
    #[command(about = "Shifting effect with color, speed, and direction")]
    Shifting(DynamicColorDirectionArgs),
    #[command(about = "Rainbow wave effect with speed and direction")]
    Wave(DynamicDirectionArgs),
    #[command(about = "Zoom effect with color and speed")]
    Zoom(DynamicColorArgs),
}

#[derive(Args, Debug)]
pub struct DynamicSpeedArgs {
    #[arg(long, required = true, help = "Effect speed from 1 to 9")]
    pub speed: u8,
}

#[derive(Args, Debug)]
pub struct DynamicColorArgs {
    #[arg(long, required = true, help = "Effect speed from 1 to 9")]
    pub speed: u8,
    #[arg(long, required = true, help = "Effect color as a 6-digit hex value")]
    pub color: String,
}

#[derive(Args, Debug)]
pub struct DynamicColorDirectionArgs {
    #[arg(long, required = true, help = "Effect speed from 1 to 9")]
    pub speed: u8,
    #[arg(long, required = true, help = "Effect color as a 6-digit hex value")]
    pub color: String,
    #[arg(
        long,
        required = true,
        help = "Effect direction: from-left or from-right"
    )]
    pub direction: Direction,
}

#[derive(Args, Debug)]
pub struct DynamicDirectionArgs {
    #[arg(long, required = true, help = "Effect speed from 1 to 9")]
    pub speed: u8,
    #[arg(
        long,
        required = true,
        help = "Effect direction: from-left or from-right"
    )]
    pub direction: Direction,
}

#[derive(Args, Debug)]
pub struct ModeArgs {
    #[arg(help = "Operation mode")]
    pub mode: OperatingMode,
    #[arg(long = "skip-whispermode", help = "Skip WhisperMode integration")]
    pub skip_whispermode: bool,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StatusTarget {
    Fan,
    Keyboard,
    Mode,
    Overdrive,
    Sound,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Direction {
    FromLeft,
    FromRight,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum OperatingMode {
    Quiet,
    Default,
    Performance,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum ToggleState {
    Enable,
    Disable,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
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

display_value!(Direction);
display_value!(OperatingMode);
display_value!(ToggleState);
display_value!(SoundPreset);
display_value!(StatusTarget);

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, error::ErrorKind};

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
    fn requires_both_fan_custom_selections() {
        assert!(Cli::try_parse_from(["resense", "fan", "custom", "--cpu", "70"]).is_err());
        assert!(
            Cli::try_parse_from(["resense", "fan", "custom", "--cpu-auto", "--gpu-auto"]).is_ok()
        );
    }

    #[test]
    fn requires_all_static_keyboard_zones() {
        assert!(
            Cli::try_parse_from([
                "resense", "keyboard", "static", "--zone1", "FF0000", "--zone2", "off", "--zone3",
                "FF0000",
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "resense", "keyboard", "static", "--zone1", "FF0000", "--zone2", "off", "--zone3",
                "FF0000", "--zone4", "off",
            ])
            .is_ok()
        );
    }

    #[test]
    fn parses_update_command() {
        let cli = Cli::try_parse_from(["resense", "update"]).unwrap();
        assert!(matches!(cli.command, Commands::Update));
    }

    #[test]
    fn parses_direct_overdrive_command() {
        let cli = Cli::try_parse_from(["resense", "overdrive", "enable"]).unwrap();

        match cli.command {
            Commands::Overdrive(args) => assert_eq!(args.state, ToggleState::Enable),
            _ => panic!("expected overdrive command"),
        }
    }

    #[test]
    fn parses_overdrive_status_target() {
        let cli = Cli::try_parse_from(["resense", "status", "overdrive", "--json"]).unwrap();

        match cli.command {
            Commands::Status(args) => {
                assert_eq!(args.target, Some(StatusTarget::Overdrive));
                assert!(args.json);
            }
            _ => panic!("expected status command"),
        }
    }

    #[test]
    fn parses_effect_specific_dynamic_commands() {
        let cli = Cli::try_parse_from([
            "resense",
            "keyboard",
            "dynamic",
            "wave",
            "--speed",
            "1",
            "--direction",
            "from-left",
        ])
        .unwrap();

        match cli.command {
            Commands::Keyboard(command) => match command.command {
                KeyboardCommands::Dynamic(dynamic) => {
                    assert!(matches!(dynamic.command, KeyboardDynamicCommands::Wave(_)));
                }
                _ => panic!("expected dynamic keyboard command"),
            },
            _ => panic!("expected keyboard command"),
        }
    }

    #[test]
    fn rejects_wave_color() {
        assert!(
            Cli::try_parse_from([
                "resense", "keyboard", "dynamic", "wave", "--color", "00FFFF",
            ])
            .is_err()
        );
    }

    #[test]
    fn rejects_incomplete_dynamic_commands() {
        assert!(Cli::try_parse_from(["resense", "keyboard", "dynamic", "wave"]).is_err());
        assert!(
            Cli::try_parse_from([
                "resense",
                "keyboard",
                "dynamic",
                "breathing",
                "--speed",
                "1"
            ])
            .is_err()
        );
    }

    #[test]
    fn clap_still_describes_the_version_flag() {
        let error = Cli::try_parse_from(["resense", "--version"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn rejects_removed_command_forms() {
        for command in [
            vec!["resense", "get", "fan"],
            vec!["resense", "fan", "speed", "--cpu", "70"],
            vec!["resense", "keyboard", "backlight-timeout", "enable"],
            vec!["resense", "sound", "--backend", "dts", "music"],
            vec!["resense", "display", "overdrive", "enable"],
        ] {
            assert!(
                Cli::try_parse_from(command.clone()).is_err(),
                "accepted {command:?}"
            );
        }
    }
}
