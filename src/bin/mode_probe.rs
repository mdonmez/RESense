#![allow(dead_code)]

mod error {
    pub type Result<T> = anyhow::Result<T>;
}

#[path = "../platform/mod.rs"]
mod platform;

use anyhow::Context;
use serde::Serialize;
use std::env;

const CMD_SET_OPERATION_MODE: u16 = 30;
const CMD_GET_GAMING_MISC_SETTING: u16 = 34;
const OPERATION_MODE_QUERY: u32 = 11;

#[derive(Debug, Serialize)]
struct Snapshot {
    requested_mode_code: u32,
    set_raw: String,
    set_return_code: u32,
    get_raw: String,
    get_value_hex: String,
    live_status: u8,
    live_mode_code: u8,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let mode_code = env::args()
        .nth(1)
        .context("usage: cargo run --bin mode_probe -- <mode_code>")?
        .parse::<u32>()
        .context("mode_code must be a u32")?;

    let (set_raw, set_return_code) =
        platform::pipe::service_set(CMD_SET_OPERATION_MODE, &[platform::pipe::u32_arg(mode_code)])?;
    let (get_raw, get_value) = platform::pipe::service_get_u64(
        CMD_GET_GAMING_MISC_SETTING,
        &[platform::pipe::u32_arg(OPERATION_MODE_QUERY)],
    )?;

    let snapshot = Snapshot {
        requested_mode_code: mode_code,
        set_raw: hex(&set_raw),
        set_return_code,
        get_raw: hex(&get_raw),
        get_value_hex: format!("{get_value:016x}"),
        live_status: (get_value & 0xFF) as u8,
        live_mode_code: ((get_value >> 8) & 0xFF) as u8,
    };

    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
