#![allow(dead_code)]

mod error {
    pub type Result<T> = anyhow::Result<T>;
}

#[path = "../platform/mod.rs"]
mod platform;

use serde::Serialize;

const CMD_GET_SYSTEM_INFORMATION: u16 = 13;
const BATTERY_BOOST_QUERY: u32 = 2;

#[derive(Debug, Serialize)]
struct Snapshot {
    raw: String,
    value_hex: String,
    status_byte: u8,
    battery_boost_byte: u8,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let (raw, value) = platform::pipe::service_get_u64(
        CMD_GET_SYSTEM_INFORMATION,
        &[platform::pipe::u32_arg(BATTERY_BOOST_QUERY)],
    )?;
    let snapshot = Snapshot {
        raw: hex(&raw),
        value_hex: format!("{value:016x}"),
        status_byte: (value & 0xFF) as u8,
        battery_boost_byte: ((value >> 40) & 0xFF) as u8,
    };
    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
