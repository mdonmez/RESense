use crate::error::Result;
use anyhow::{Context, bail};
use std::io::{Read, Write};
use std::time::Duration;

pub const SERVICE_PIPE_NAME: &str = r"\\.\pipe\PredatorSense_service_namedpipe";
pub const ADMIN_PIPE_PREFIX: &str = r"\\.\pipe\PredatorSense_admin_agent_";

#[derive(Debug, Clone, Copy)]
pub(crate) enum Argument<'a> {
    U32(u32),
    U64(u64),
    Utf16(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SetReply {
    pub return_code: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct PipeClient {
    name: String,
}

impl PipeClient {
    pub(crate) fn service() -> Self {
        Self::new(SERVICE_PIPE_NAME)
    }

    pub(crate) fn admin(session_id: u32) -> Self {
        Self::new(&format!("{ADMIN_PIPE_PREFIX}{session_id}"))
    }

    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub(crate) fn set(&self, command: u16, args: &[Argument<'_>]) -> Result<SetReply> {
        let reply = self.request(command, args, 9)?;
        Ok(SetReply {
            return_code: u32::from_le_bytes(reply[5..9].try_into()?),
        })
    }

    pub(crate) fn get_u64(&self, command: u16, args: &[Argument<'_>]) -> Result<u64> {
        let reply = self.request(command, args, 13)?;
        Ok(u64::from_le_bytes(reply[5..13].try_into()?))
    }

    pub(crate) fn get_u32(&self, command: u16, args: &[Argument<'_>]) -> Result<u32> {
        let reply = self.request(command, args, 9)?;
        Ok(u32::from_le_bytes(reply[5..9].try_into()?))
    }

    pub(crate) fn fire(&self, command: u16, args: &[Argument<'_>]) -> Result<()> {
        let request = build_message(command, args)?;
        let mut pipe = open(&self.name)?;
        pipe.write_all(&request)
            .with_context(|| format!("writing command {command} to {}", self.name))?;
        Ok(())
    }

    fn request(&self, command: u16, args: &[Argument<'_>], reply_size: usize) -> Result<Vec<u8>> {
        let request = build_message(command, args)?;
        let mut pipe = open(&self.name)?;
        pipe.write_all(&request)
            .with_context(|| format!("writing command {command} to {}", self.name))?;
        let mut reply = vec![0u8; reply_size];
        pipe.read_exact(&mut reply)
            .with_context(|| format!("reading command {command} from {}", self.name))?;
        if reply.len() != reply_size {
            bail!(
                "invalid reply length for command {command}: expected {reply_size}, got {}",
                reply.len()
            );
        }
        Ok(reply)
    }
}

pub(crate) fn build_message(command: u16, args: &[Argument<'_>]) -> Result<Vec<u8>> {
    if args.len() > u8::MAX as usize {
        bail!("too many pipe arguments: {}", args.len());
    }

    let payload_len = args.iter().map(|arg| arg.encoded_len()).sum::<usize>();
    let mut message = Vec::with_capacity(3 + args.len() * 4 + payload_len);
    message.extend_from_slice(&command.to_le_bytes());
    message.push(args.len() as u8);
    for arg in args {
        message.extend_from_slice(&(arg.encoded_len() as u32).to_le_bytes());
        arg.append_to(&mut message);
    }
    Ok(message)
}

impl Argument<'_> {
    fn encoded_len(self) -> usize {
        match self {
            Self::U32(_) => 4,
            Self::U64(_) => 8,
            Self::Utf16(value) => (value.encode_utf16().count() + 1) * 2,
        }
    }

    fn append_to(self, destination: &mut Vec<u8>) {
        match self {
            Self::U32(value) => destination.extend_from_slice(&value.to_le_bytes()),
            Self::U64(value) => destination.extend_from_slice(&value.to_le_bytes()),
            Self::Utf16(value) => {
                for unit in value.encode_utf16().chain(std::iter::once(0)) {
                    destination.extend_from_slice(&unit.to_le_bytes());
                }
            }
        }
    }
}

fn open(name: &str) -> Result<std::fs::File> {
    let mut last_error = None;
    for _ in 0..10 {
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(name)
        {
            Ok(file) => return Ok(file),
            Err(error) => {
                let busy = error.raw_os_error() == Some(231);
                last_error = Some(error);
                if !busy {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let error = last_error.expect("pipe open loop always records an error");
    Err(anyhow::anyhow!(
        "failed to open PredatorSense service; ensure PSSvc is running and NitroSense is installed ({error})"
    ))
}

#[cfg(feature = "dev-tools")]
pub(crate) fn raw_request(
    pipe: &PipeClient,
    command: u16,
    args: &[Argument<'_>],
    reply_size: usize,
) -> Result<Vec<u8>> {
    pipe.request(command, args, reply_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_typed_pipe_message_without_argument_buffers() {
        let message = build_message(27, &[Argument::U64(123)]).unwrap();
        assert_eq!(&message[0..2], &27u16.to_le_bytes());
        assert_eq!(message[2], 1);
        assert_eq!(&message[3..7], &8u32.to_le_bytes());
        assert_eq!(&message[7..15], &123u64.to_le_bytes());
    }

    #[test]
    fn encodes_utf16_arguments() {
        let message = build_message(3, &[Argument::Utf16("A")]).unwrap();
        assert_eq!(&message[7..11], &[b'A', 0, 0, 0]);
    }
}
