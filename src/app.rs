use crate::cli::{
    Cli, Commands, Direction as CliDirection, FanCommands, KeyboardCommands, KeyboardDynamicMode,
    StatusTarget,
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
            FanCommands::Auto => output::print_fan_state(device.set_fan_auto()?),
            FanCommands::Max => output::print_fan_state(device.set_fan_max()?),
            FanCommands::Custom(args) => {
                let request = FanCustomRequest::new(
                    fan_change(args.cpu, args.cpu_auto, "cpu")?,
                    fan_change(args.gpu, args.gpu_auto, "gpu")?,
                )?;
                output::print_fan_state(device.set_fan_custom(request)?)
            }
        },
        Commands::Keyboard(command) => match command.command {
            KeyboardCommands::Brightness(args) => output::print_keyboard_state(
                device.set_keyboard_brightness(Brightness::new(args.level)?)?,
            ),
            KeyboardCommands::Timeout(args) => {
                output::print_keyboard_state(device.set_keyboard_timeout(args.state.enabled())?)
            }
            KeyboardCommands::Static(args) => {
                output::print_keyboard_state(device.set_keyboard_static(static_request(args)?)?)
            }
            KeyboardCommands::Dynamic(args) => {
                output::print_keyboard_state(device.set_keyboard_dynamic(dynamic_request(args)?)?)
            }
            KeyboardCommands::Sticky(args) => {
                output::print_keyboard_state(device.set_sticky_keys(args.state.enabled())?)
            }
            KeyboardCommands::WinMenu(args) => {
                output::print_keyboard_state(device.set_win_menu_lock(args.state.enabled())?)
            }
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

fn dynamic_request(args: crate::cli::KeyboardDynamicArgs) -> Result<DynamicRequest> {
    DynamicRequest::new(
        dynamic_mode(args.mode),
        args.speed.map(DynamicSpeed::new).transpose()?,
        args.color.as_deref().map(Rgb::parse).transpose()?,
        args.direction.map(direction),
    )
}

fn dynamic_mode(mode: KeyboardDynamicMode) -> DynamicMode {
    match mode {
        KeyboardDynamicMode::Breathing => DynamicMode::Breathing,
        KeyboardDynamicMode::Neon => DynamicMode::Neon,
        KeyboardDynamicMode::Shifting => DynamicMode::Shifting,
        KeyboardDynamicMode::Wave => DynamicMode::Wave,
        KeyboardDynamicMode::Zoom => DynamicMode::Zoom,
    }
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
