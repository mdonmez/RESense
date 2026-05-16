use crate::error::Result;
use crate::platform::service;
use anyhow::{Context, bail};

pub const SERVICE_PIPE_NAME: &str = r"\\.\pipe\PredatorSense_service_namedpipe";
pub const ADMIN_PIPE_PREFIX: &str = r"\\.\pipe\PredatorSense_admin_agent_";
pub const SET_REPLY_SIZE: usize = 9;
pub const GET_REPLY_SIZE: usize = 13;

pub fn build_message(cmd_code: u16, args: &[Vec<u8>]) -> Result<Vec<u8>> {
    if args.len() > u8::MAX as usize {
        bail!("too many pipe arguments: {}", args.len());
    }

    let mut payload = Vec::new();
    payload.extend_from_slice(&cmd_code.to_le_bytes());
    payload.push(args.len() as u8);
    for arg in args {
        payload.extend_from_slice(&(arg.len() as u32).to_le_bytes());
        payload.extend_from_slice(arg);
    }
    Ok(payload)
}

pub fn u32_arg(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

pub fn u64_arg(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

pub fn utf16_string_arg(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

pub fn service_set(cmd_code: u16, args: &[Vec<u8>]) -> Result<(Vec<u8>, u32)> {
    send_set(SERVICE_PIPE_NAME, cmd_code, args, SET_REPLY_SIZE)
}

pub fn service_get_u64(cmd_code: u16, args: &[Vec<u8>]) -> Result<(Vec<u8>, u64)> {
    send_get_u64(SERVICE_PIPE_NAME, cmd_code, args, GET_REPLY_SIZE)
}

pub fn send_set(
    pipe_name: &str,
    cmd_code: u16,
    args: &[Vec<u8>],
    reply_size: usize,
) -> Result<(Vec<u8>, u32)> {
    let raw = send_with_reply(pipe_name, cmd_code, args, reply_size)?;
    if raw.len() < 9 {
        bail!("set reply too short: {} bytes", raw.len());
    }
    Ok((raw.clone(), u32::from_le_bytes(raw[5..9].try_into()?)))
}

pub fn send_get_u64(
    pipe_name: &str,
    cmd_code: u16,
    args: &[Vec<u8>],
    reply_size: usize,
) -> Result<(Vec<u8>, u64)> {
    let raw = send_with_reply(pipe_name, cmd_code, args, reply_size)?;
    if raw.len() < 13 {
        bail!("get reply too short: {} bytes", raw.len());
    }
    Ok((raw.clone(), u64::from_le_bytes(raw[5..13].try_into()?)))
}

pub fn send_fire_and_forget(pipe_name: &str, cmd_code: u16, args: &[Vec<u8>]) -> Result<()> {
    ensure_transport_available(pipe_name)?;
    let request = build_message(cmd_code, args)?;
    platform::write_only(pipe_name, &request).with_context(|| format!("writing to {pipe_name}"))
}

fn send_with_reply(
    pipe_name: &str,
    cmd_code: u16,
    args: &[Vec<u8>],
    reply_size: usize,
) -> Result<Vec<u8>> {
    ensure_transport_available(pipe_name)?;
    let request = build_message(cmd_code, args)?;
    platform::write_read(pipe_name, &request, reply_size)
        .with_context(|| format!("pipe command {cmd_code} on {pipe_name}"))
}

fn ensure_transport_available(pipe_name: &str) -> Result<()> {
    if uses_predator_transport(pipe_name) {
        service::ensure_predator_service_running()?;
    }
    Ok(())
}

fn uses_predator_transport(pipe_name: &str) -> bool {
    pipe_name.contains("PredatorSense_") || pipe_name.contains("predatorsense_")
}

#[cfg(windows)]
mod platform {
    use crate::error::Result;
    use anyhow::bail;
    use std::{ptr, thread, time::Duration};
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, OPEN_EXISTING, ReadFile, WriteFile,
    };

    const ERROR_PIPE_BUSY: u32 = 231;

    pub fn write_read(pipe_name: &str, request: &[u8], reply_size: usize) -> Result<Vec<u8>> {
        let handle = Handle::open(pipe_name)?;
        handle.write_all(request)?;
        handle.read_exact(reply_size)
    }

    pub fn write_only(pipe_name: &str, request: &[u8]) -> Result<()> {
        let handle = Handle::open(pipe_name)?;
        handle.write_all(request)
    }

    struct Handle(HANDLE);

    impl Handle {
        fn open(pipe_name: &str) -> Result<Self> {
            let wide: Vec<u16> = pipe_name.encode_utf16().chain(std::iter::once(0)).collect();
            let mut last_error = 0;
            for _ in 0..10 {
                let handle = unsafe {
                    CreateFileW(
                        wide.as_ptr(),
                        FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                        0,
                        ptr::null(),
                        OPEN_EXISTING,
                        0,
                        ptr::null_mut(),
                    )
                };
                if handle != INVALID_HANDLE_VALUE {
                    return Ok(Self(handle));
                }
                last_error = unsafe { GetLastError() };
                if last_error != ERROR_PIPE_BUSY {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
            bail!("failed to open {pipe_name}: Windows error {last_error}");
        }

        fn write_all(&self, request: &[u8]) -> Result<()> {
            let mut written = 0u32;
            let ok = unsafe {
                WriteFile(
                    self.0,
                    request.as_ptr().cast(),
                    request.len() as u32,
                    &mut written,
                    ptr::null_mut(),
                )
            };
            if ok == 0 {
                bail!("WriteFile failed: Windows error {}", unsafe {
                    GetLastError()
                });
            }
            if written as usize != request.len() {
                bail!("short pipe write: wrote {written} of {}", request.len());
            }
            Ok(())
        }

        fn read_exact(&self, reply_size: usize) -> Result<Vec<u8>> {
            let mut reply = vec![0u8; reply_size];
            let mut read = 0u32;
            let ok = unsafe {
                ReadFile(
                    self.0,
                    reply.as_mut_ptr().cast(),
                    reply.len() as u32,
                    &mut read,
                    ptr::null_mut(),
                )
            };
            if ok == 0 {
                bail!("ReadFile failed: Windows error {}", unsafe {
                    GetLastError()
                });
            }
            if read as usize != reply_size {
                bail!("unexpected reply size: {read}");
            }
            Ok(reply)
        }
    }

    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::Result;
    use anyhow::bail;

    pub fn write_read(_pipe_name: &str, _request: &[u8], _reply_size: usize) -> Result<Vec<u8>> {
        bail!("RESense hardware transport is only available on Windows");
    }

    pub fn write_only(_pipe_name: &str, _request: &[u8]) -> Result<()> {
        bail!("RESense hardware transport is only available on Windows");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_pipe_message() {
        let message = build_message(27, &[u64_arg(123)]).unwrap();
        assert_eq!(&message[0..2], &27u16.to_le_bytes());
        assert_eq!(message[2], 1);
        assert_eq!(&message[3..7], &8u32.to_le_bytes());
        assert_eq!(&message[7..15], &123u64.to_le_bytes());
    }

    #[test]
    fn detects_predator_transport_names() {
        assert!(uses_predator_transport(SERVICE_PIPE_NAME));
        assert!(uses_predator_transport(
            r"\\.\pipe\PredatorSense_admin_agent_1"
        ));
        assert!(uses_predator_transport(
            r"\\.\pipe\predatorsense_service_namedpipe"
        ));
        assert!(!uses_predator_transport(r"\\.\pipe\something_else"));
    }
}
