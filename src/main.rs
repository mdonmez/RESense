mod cli;
mod error;
mod nitrosense;
mod platform;

use clap::Parser;
use cli::*;
use error::{Result, validate};
use std::thread;
use std::time::Duration;

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
            if args.interval == Some(0) {
                anyhow::bail!("interval must be greater than zero seconds");
            }
            let interval = Duration::from_secs(args.interval.unwrap_or(2));
            loop {
                let status = nitrosense::status::read_status(args.target);
                if args.json {
                    if args.watch {
                        println!("{}", serde_json::to_string(&status)?);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    }
                } else {
                    nitrosense::status::print_text(&status, args.target);
                }

                if !args.watch {
                    break;
                }
                thread::sleep(interval);
            }
        }
        Commands::Fan(command) => match command.command {
            FanCommands::Auto => {
                nitrosense::policy::ensure_fan_control_allowed()?;
                nitrosense::fan::set_mode(FanMode::Auto)?;
                println!("fan.mode=auto");
            }
            FanCommands::Max => {
                nitrosense::policy::ensure_fan_control_allowed()?;
                nitrosense::fan::set_mode(FanMode::Max)?;
                println!("fan.mode=max");
            }
            FanCommands::Custom(args) => {
                validate::fan_custom_args(&args)?;
                nitrosense::policy::ensure_fan_control_allowed()?;
                let custom = nitrosense::fan::set_custom(&args)?;
                println!("fan.mode=custom");
                nitrosense::status::print_state_text("fan.custom", &custom)?;
            }
        },
        Commands::Keyboard(command) => match command.command {
            KeyboardCommands::Brightness(args) => {
                validate::range("brightness", args.level, 1, 5)?;
                nitrosense::keyboard::set_brightness(args.level)?;
                println!("keyboard.brightness={}", args.level);
            }
            KeyboardCommands::Timeout(args) => {
                nitrosense::display::set_backlight_timeout(args.state.enabled())?;
                println!("keyboard.backlight_timeout={}", args.state.enabled());
            }
            KeyboardCommands::Static(args) => {
                validate::static_args(&args)?;
                let state = nitrosense::keyboard::set_static(&args)?;
                nitrosense::status::print_state_text("keyboard", &state)?;
            }
            KeyboardCommands::Dynamic(args) => {
                validate::dynamic_args(&args)?;
                let state = nitrosense::keyboard::set_dynamic(&args)?;
                nitrosense::status::print_state_text("keyboard", &state)?;
            }
            KeyboardCommands::Sticky(args) => {
                nitrosense::keyboard::set_sticky_keys(args.state.enabled())?;
                println!("keyboard.sticky={}", args.state.enabled());
            }
            KeyboardCommands::WinMenu(args) => {
                nitrosense::keyboard::set_win_menu_lock(args.state.enabled())?;
                println!("keyboard.win_menu={}", args.state.enabled());
            }
        },
        Commands::Mode(args) => {
            nitrosense::policy::enforce_operation_mode_fan_policy(args.mode)?;
            nitrosense::mode::set_operation_mode(args.mode, args.skip_whispermode)?;
            nitrosense::policy::enforce_operation_mode_fan_policy(args.mode)?;
            println!("mode={}", args.mode);
        }
        Commands::Display(command) => match command.command {
            DisplayCommands::Overdrive(args) => {
                nitrosense::display::set_overdrive(args.state.enabled())?;
                println!("display.overdrive={}", args.state.enabled());
            }
        },
        Commands::Sound(args) => {
            nitrosense::sound::set_preset(args.preset)?;
            println!("sound.preset={}", args.preset);
        }
    }

    Ok(())
}
