use crate::cli::{
    Cli, Commands, Direction as CliDirection, FanCommands, KeyboardCommands,
    KeyboardDynamicCommands, StatusTarget,
};
use crate::device::{
    Brightness, Device, Direction, DynamicMode, DynamicRequest, DynamicSpeed, FanChange,
    FanCustomRequest, OperationMode, Percent, Rgb, SoundPreset, StaticRequest, ZoneChange,
};
use crate::error::Result;
use crate::output::{self, StatusValue};
use anyhow::bail;
use std::thread;
use std::time::Duration;

pub fn run(cli: Cli) -> Result<()> {
    let allow_any_model = cli.dangerously_allow_any_model;
    let command = cli.command;
    match command {
        Commands::Update => crate::update::run_update(),
        command => {
            let device = Device::connect(allow_any_model)?;
            run_hardware_command(device, command)
        }
    }
}

fn run_hardware_command(device: Device, command: Commands) -> Result<()> {
    match command {
        Commands::Status(args) => {
            run_status(&device, args.target, args.json, args.watch, args.interval)?
        }
        Commands::Fan(command) => match command.command {
            FanCommands::Auto => output::print_fan_mode(device.set_fan_auto()?.mode),
            FanCommands::Max => output::print_fan_mode(device.set_fan_max()?.mode),
            FanCommands::Custom(args) => {
                let request = FanCustomRequest::new(
                    fan_change(args.cpu, args.cpu_auto, "cpu")?,
                    fan_change(args.gpu, args.gpu_auto, "gpu")?,
                )?;
                output::print_fan_custom_state(device.set_fan_custom(request)?, request)
            }
        },
        Commands::Keyboard(command) => match command.command {
            KeyboardCommands::Brightness(args) => output::print_keyboard_brightness(
                device.set_keyboard_brightness(Brightness::new(args.level)?)?,
            ),
            KeyboardCommands::Timeout(args) => {
                output::print_keyboard_timeout(device.set_keyboard_timeout(args.state.enabled())?)
            }
            KeyboardCommands::Static(args) => {
                output::print_keyboard_lighting(device.set_keyboard_static(static_request(args)?)?)
            }
            KeyboardCommands::Dynamic(args) => output::print_keyboard_lighting(
                device.set_keyboard_dynamic(dynamic_request(args.command)?)?,
            ),
            KeyboardCommands::Sticky(args) => {
                output::print_keyboard_sticky_keys(device.set_sticky_keys(args.state.enabled())?)
            }
            KeyboardCommands::WinMenu(args) => output::print_keyboard_win_menu_lock(
                device.set_win_menu_lock(args.state.enabled())?,
            ),
        },
        Commands::Mode(args) => {
            output::print_mode(device.set_mode(operation_mode(args.mode), args.skip_whispermode)?)
        }
        Commands::Display(command) => match command.command {
            crate::cli::DisplayCommands::Overdrive(args) => {
                output::print_display(device.set_display_overdrive(args.state.enabled())?)
            }
        },
        Commands::Sound(args) => output::print_sound(device.set_sound(sound_preset(args.preset))?),
        Commands::Update => unreachable!("update is handled before connecting the device"),
    }
    Ok(())
}

fn run_status(
    device: &Device,
    target: Option<StatusTarget>,
    json: bool,
    watch: bool,
    interval: Option<u64>,
) -> Result<()> {
    if interval == Some(0) {
        bail!("interval must be greater than zero seconds")
    }
    let interval = Duration::from_secs(interval.unwrap_or(2));
    loop {
        let value = read_status(device, target)?;
        output::print_status(value, target, json, watch)?;
        if !watch {
            break;
        }
        thread::sleep(interval);
    }
    Ok(())
}

fn read_status(device: &Device, target: Option<StatusTarget>) -> Result<StatusValue> {
    Ok(match target {
        None => StatusValue::All(device.status()?),
        Some(StatusTarget::Fan) => StatusValue::Fan(device.fan()?),
        Some(StatusTarget::Keyboard) => StatusValue::Keyboard(device.keyboard()?),
        Some(StatusTarget::Mode) => StatusValue::Mode(device.mode()?),
        Some(StatusTarget::Display) => StatusValue::Display(device.display_overdrive()?),
        Some(StatusTarget::Sound) => StatusValue::Sound(device.sound()?),
    })
}

fn fan_change(value: Option<u8>, auto: bool, name: &str) -> Result<Option<FanChange>> {
    match (value, auto) {
        (Some(_), true) => bail!("choose either --{name} or --{name}-auto"),
        (Some(value), false) => Ok(Some(FanChange::Manual(Percent::new(value)?))),
        (None, true) => Ok(Some(FanChange::Auto)),
        (None, false) => Ok(None),
    }
}

fn static_request(args: crate::cli::KeyboardStaticArgs) -> Result<StaticRequest> {
    StaticRequest::new([
        zone_change(args.zone1.as_deref())?,
        zone_change(args.zone2.as_deref())?,
        zone_change(args.zone3.as_deref())?,
        zone_change(args.zone4.as_deref())?,
    ])
}

fn zone_change(value: Option<&str>) -> Result<Option<ZoneChange>> {
    match value {
        None => Ok(None),
        Some(value) if value.trim().eq_ignore_ascii_case("off") => Ok(Some(ZoneChange::Off)),
        Some(value) => Ok(Some(ZoneChange::Color(Rgb::parse(value)?))),
    }
}

fn dynamic_request(command: KeyboardDynamicCommands) -> Result<DynamicRequest> {
    match command {
        KeyboardDynamicCommands::Breathing(args) => {
            dynamic_request_from_args(DynamicMode::Breathing, args.speed, Some(args.color), None)
        }
        KeyboardDynamicCommands::Neon(args) => {
            dynamic_request_from_args(DynamicMode::Neon, args.speed, None, None)
        }
        KeyboardDynamicCommands::Shifting(args) => dynamic_request_from_args(
            DynamicMode::Shifting,
            args.speed,
            Some(args.color),
            Some(args.direction),
        ),
        KeyboardDynamicCommands::Wave(args) => {
            dynamic_request_from_args(DynamicMode::Wave, args.speed, None, Some(args.direction))
        }
        KeyboardDynamicCommands::Zoom(args) => {
            dynamic_request_from_args(DynamicMode::Zoom, args.speed, Some(args.color), None)
        }
    }
}

fn dynamic_request_from_args(
    mode: DynamicMode,
    speed: u8,
    color: Option<String>,
    direction_value: Option<CliDirection>,
) -> Result<DynamicRequest> {
    DynamicRequest::new(
        mode,
        DynamicSpeed::new(speed)?,
        color.as_deref().map(Rgb::parse).transpose()?,
        direction_value.map(direction),
    )
}

fn direction(value: CliDirection) -> Direction {
    match value {
        CliDirection::FromLeft => Direction::FromLeft,
        CliDirection::FromRight => Direction::FromRight,
    }
}

fn operation_mode(value: crate::cli::OperatingMode) -> OperationMode {
    match value {
        crate::cli::OperatingMode::Quiet => OperationMode::Quiet,
        crate::cli::OperatingMode::Default => OperationMode::Default,
        crate::cli::OperatingMode::Performance => OperationMode::Performance,
    }
}

fn sound_preset(value: crate::cli::SoundPreset) -> SoundPreset {
    match value {
        crate::cli::SoundPreset::Music => SoundPreset::Music,
        crate::cli::SoundPreset::Movies => SoundPreset::Movies,
        crate::cli::SoundPreset::Voice => SoundPreset::Voice,
        crate::cli::SoundPreset::Strategy => SoundPreset::Strategy,
        crate::cli::SoundPreset::Rpg => SoundPreset::Rpg,
        crate::cli::SoundPreset::Shooter => SoundPreset::Shooter,
        crate::cli::SoundPreset::Custom => SoundPreset::Custom,
        crate::cli::SoundPreset::Auto => SoundPreset::Auto,
    }
}
