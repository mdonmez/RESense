use crate::error::Result;
use crate::platform::Platform;
use crate::platform::pipe::{self, Argument, PipeClient};
use anyhow::{Context, bail};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ProbeResult {
    transport: String,
    command: u16,
    value: Option<u64>,
    status_byte: Option<u8>,
    reply_hex: String,
}

pub fn run<I>(arguments: I) -> Result<()>
where
    I: IntoIterator<Item = String>,
{
    let mut arguments = arguments.into_iter();
    match arguments.next().as_deref() {
        Some("query") => query(&mut arguments),
        Some("set-mode") => set_mode(&mut arguments),
        Some("help") | None => {
            println!(
                "usage: probe query <service|admin> <session-id|current> <command> [u32-argument] [reply-size]\n       probe set-mode <0|1|4>"
            );
            Ok(())
        }
        Some(command) => bail!("unknown developer probe {command}"),
    }
}

fn query(arguments: &mut impl Iterator<Item = String>) -> Result<()> {
    let transport = arguments.next().context("missing transport")?;
    let session = arguments.next().context("missing session")?;
    let command: u16 = arguments
        .next()
        .context("missing command")?
        .parse()
        .context("command must be a u16")?;
    let argument = arguments
        .next()
        .map(|value| value.parse::<u32>())
        .transpose()?;
    let reply_size = arguments
        .next()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(13);

    let _platform = Platform::connect(true)?;
    let pipe = match transport.as_str() {
        "service" => PipeClient::service(),
        "admin" => PipeClient::admin(resolve_session(&session)?),
        _ => bail!("transport must be service or admin"),
    };
    let args = argument.map_or_else(Vec::new, |value| vec![Argument::U32(value)]);
    let raw = pipe::raw_request(&pipe, command, &args, reply_size)?;
    let value = if raw.len() >= 13 {
        Some(u64::from_le_bytes(raw[5..13].try_into()?))
    } else if raw.len() >= 9 {
        Some(u32::from_le_bytes(raw[5..9].try_into()?) as u64)
    } else {
        None
    };
    let status_byte = value.map(|value| value as u8);
    let result = ProbeResult {
        transport,
        command,
        value,
        status_byte,
        reply_hex: hex(&raw),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn set_mode(arguments: &mut impl Iterator<Item = String>) -> Result<()> {
    let code: u32 = arguments
        .next()
        .context("missing operation mode code")?
        .parse()
        .context("operation mode code must be a u32")?;
    let platform = Platform::connect(true)?;
    let return_code = platform.service_set(30, &[Argument::U32(code)])?;
    println!("{{\"return_code\":{return_code}}}");
    Ok(())
}

fn resolve_session(value: &str) -> Result<u32> {
    if value == "current" {
        return crate::platform::current_session_id();
    }
    Ok(value.parse()?)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
