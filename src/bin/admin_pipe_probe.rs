#![allow(dead_code)]

mod error {
    pub type Result<T> = anyhow::Result<T>;
}

#[path = "../platform/mod.rs"]
mod platform;

use anyhow::{Context, bail};
use serde::Serialize;
use std::env;

const CMD_ADMIN_SET_STICKY_KEYS: u16 = 2;
const CMD_GET_DTS_SOUND_MODE: u16 = 13;
const CMD_SET_DTS_SOUND_MODE: u16 = 14;
const CMD_ADMIN_SET_WHISPERMODE: u16 = 15;
const ADMIN_GET_REPLY_SIZE: usize = 9;

#[derive(Debug, Serialize)]
struct GetSnapshot {
    pipe_name: String,
    cmd_code: u16,
    raw: String,
    value_u32: u32,
    value_i32: i32,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let session_id = env::args()
        .nth(1)
        .context("usage: cargo run --bin admin_pipe_probe -- <session_id> <action> [value]")?
        .parse::<u32>()
        .context("session_id must be a u32")?;
    let action = env::args()
        .nth(2)
        .context("missing action: sound-get | sound-set | sticky-set | whisper-set")?;

    let pipe_name = platform::session::admin_pipe_name(session_id);

    match action.as_str() {
        "sound-get" => {
            let (raw, value_u32) =
                platform::pipe::send_set(&pipe_name, CMD_GET_DTS_SOUND_MODE, &[], ADMIN_GET_REPLY_SIZE)?;
            let snapshot = GetSnapshot {
                pipe_name,
                cmd_code: CMD_GET_DTS_SOUND_MODE,
                raw: hex(&raw),
                value_u32,
                value_i32: i32::from_le_bytes(value_u32.to_le_bytes()),
            };
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
            Ok(())
        }
        "sound-set" => {
            let value = parse_u32_arg(3, "sound-set requires a preset code")?;
            platform::pipe::send_fire_and_forget(
                &pipe_name,
                CMD_SET_DTS_SOUND_MODE,
                &[platform::pipe::u32_arg(value)],
            )?;
            println!("ok pipe={pipe_name} cmd={CMD_SET_DTS_SOUND_MODE} value={value}");
            Ok(())
        }
        "sticky-set" => {
            let enabled = parse_bool_arg(3, "sticky-set requires 0 or 1")?;
            platform::pipe::send_fire_and_forget(
                &pipe_name,
                CMD_ADMIN_SET_STICKY_KEYS,
                &[platform::pipe::u32_arg(enabled as u32)],
            )?;
            println!("ok pipe={pipe_name} cmd={CMD_ADMIN_SET_STICKY_KEYS} enabled={enabled}");
            Ok(())
        }
        "whisper-set" => {
            let enabled = parse_bool_arg(3, "whisper-set requires 0 or 1")?;
            platform::pipe::send_fire_and_forget(
                &pipe_name,
                CMD_ADMIN_SET_WHISPERMODE,
                &[platform::pipe::u32_arg(enabled as u32)],
            )?;
            println!("ok pipe={pipe_name} cmd={CMD_ADMIN_SET_WHISPERMODE} enabled={enabled}");
            Ok(())
        }
        _ => bail!("unknown action {action}"),
    }
}

fn parse_u32_arg(index: usize, help: &str) -> anyhow::Result<u32> {
    env::args()
        .nth(index)
        .with_context(|| help.to_string())?
        .parse::<u32>()
        .context("value must be a u32")
}

fn parse_bool_arg(index: usize, help: &str) -> anyhow::Result<bool> {
    match parse_u32_arg(index, help)? {
        0 => Ok(false),
        1 => Ok(true),
        other => bail!("expected 0 or 1, got {other}"),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
