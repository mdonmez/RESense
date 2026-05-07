mod cli;
mod error;
mod nitrosense;
mod platform;

use clap::Parser;
use cli::*;
use error::{Result, validate};

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    platform::model::ensure_supported_model(cli.dangerously_allow_any_model)?;

    match cli.command {
        Commands::Status(args) => {
            let status = nitrosense::status::read_status();
            if args.json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                nitrosense::status::print_text(&status);
            }
        }
        Commands::Fan(command) => match command.command {
            FanCommands::Mode(args) => {
                nitrosense::policy::ensure_fan_control_allowed()?;
                nitrosense::fan::set_mode(args.mode)?;
                println!("fan_mode={}", args.mode);
            }
            FanCommands::Speed(args) => {
                validate::fan_speed_args(&args)?;
                nitrosense::policy::ensure_fan_control_allowed()?;
                let result = nitrosense::fan::set_speed(&args)?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },
        Commands::Keyboard(command) => match command.command {
            KeyboardCommands::Brightness(args) => {
                validate::range("brightness", args.level, 1, 5)?;
                nitrosense::keyboard::set_brightness(args.level)?;
                println!("keyboard_brightness={}", args.level);
            }
            KeyboardCommands::Static(args) => {
                validate::static_args(&args)?;
                let state = nitrosense::keyboard::set_static(&args)?;
                println!("{}", serde_json::to_string_pretty(&state)?);
            }
            KeyboardCommands::Dynamic(args) => {
                validate::dynamic_args(&args)?;
                let state = nitrosense::keyboard::set_dynamic(&args)?;
                println!("{}", serde_json::to_string_pretty(&state)?);
            }
            KeyboardCommands::Sticky(args) => {
                nitrosense::keyboard::set_sticky_keys(args.state.enabled())?;
                println!("sticky_keys={}", args.state);
            }
            KeyboardCommands::WinMenu(args) => {
                nitrosense::keyboard::set_win_menu_lock(args.state.enabled())?;
                println!("win_menu_key_lock={}", args.state);
            }
        },
        Commands::Mode(args) => {
            nitrosense::policy::enforce_operation_mode_fan_policy(args.mode)?;
            nitrosense::mode::set_operation_mode(args.mode, args.skip_whispermode)?;
            nitrosense::policy::enforce_operation_mode_fan_policy(args.mode)?;
            println!("operation_mode={}", args.mode);
        }
        Commands::Display(command) => match command.command {
            DisplayCommands::Overdrive(args) => {
                nitrosense::display::set_overdrive(args.state.enabled())?;
                println!("display_overdrive={}", args.state);
            }
            DisplayCommands::BacklightTimeout(args) => {
                if let Some(percent) = args.brightness_percent {
                    validate::range("brightness-percent", percent, 0, 100)?;
                }
                nitrosense::display::set_backlight_timeout(
                    args.state.enabled(),
                    args.brightness_percent,
                )?;
                println!("backlight_timeout={}", args.state);
            }
        },
        Commands::Sound(args) => {
            nitrosense::sound::set_preset(args.backend.unwrap_or(SoundBackend::Auto), args.preset)?;
            println!("sound_preset={}", args.preset);
        }
    }

    Ok(())
}
