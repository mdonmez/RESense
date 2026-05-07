use crate::error::Result;

const PREDATOR_SERVICE_NAME: &str = "PSSvc";
const PREDATOR_SERVICE_DISPLAY_NAME: &str = "Predator Service";

pub fn ensure_predator_service_running() -> Result<()> {
    platform::ensure_predator_service_running()
}

#[cfg(windows)]
mod platform {
    use crate::error::Result;
    use anyhow::bail;
    use std::ptr;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::Services::{
        CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, SC_HANDLE,
        SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_CONTINUE_PENDING, SERVICE_DISABLED,
        SERVICE_PAUSE_PENDING, SERVICE_PAUSED, SERVICE_QUERY_CONFIG, SERVICE_QUERY_STATUS,
        SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS_PROCESS, SERVICE_STOP_PENDING,
        SERVICE_STOPPED,
    };

    use super::{PREDATOR_SERVICE_DISPLAY_NAME, PREDATOR_SERVICE_NAME};

    pub fn ensure_predator_service_running() -> Result<()> {
        let manager = ServiceHandle::open_manager()?;
        let service = ServiceHandle::open_service(manager.0, PREDATOR_SERVICE_NAME)?;
        let status = service.query_status()?;

        if status.dwCurrentState == SERVICE_RUNNING || status.dwCurrentState == SERVICE_START_PENDING
        {
            return Ok(());
        }

        let startup_type = service.query_startup_type().ok();
        let state_name = service_state_name(status.dwCurrentState);
        let startup_note = match startup_type {
            Some(SERVICE_DISABLED) => " It is disabled in Windows Services.",
            Some(_) => "",
            None => "",
        };

        bail!(
            "{display_name} ({service_name}) is not running. Current state: {state_name}.{startup_note} Start the NitroSense/Predator service and try again",
            display_name = PREDATOR_SERVICE_DISPLAY_NAME,
            service_name = PREDATOR_SERVICE_NAME,
            state_name = state_name,
            startup_note = startup_note
        );
    }

    fn service_state_name(state: u32) -> &'static str {
        match state {
            SERVICE_STOPPED => "stopped",
            SERVICE_STOP_PENDING => "stop_pending",
            SERVICE_RUNNING => "running",
            SERVICE_START_PENDING => "start_pending",
            SERVICE_CONTINUE_PENDING => "continue_pending",
            SERVICE_PAUSE_PENDING => "pause_pending",
            SERVICE_PAUSED => "paused",
            _ => "unknown",
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    struct ServiceHandle(SC_HANDLE);

    impl ServiceHandle {
        fn open_manager() -> Result<Self> {
            let handle =
                unsafe { OpenSCManagerW(ptr::null(), ptr::null(), SC_MANAGER_CONNECT) };
            if handle.is_null() {
                bail!(
                    "OpenSCManagerW failed while checking Predator service: Windows error {}",
                    unsafe { GetLastError() }
                );
            }
            Ok(Self(handle))
        }

        fn open_service(manager: SC_HANDLE, name: &str) -> Result<Self> {
            let name_w = wide(name);
            let handle = unsafe {
                OpenServiceW(
                    manager,
                    name_w.as_ptr(),
                    SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
                )
            };
            if handle.is_null() {
                bail!(
                    "{} ({}) is not installed or could not be opened. Install or repair NitroSense/Predator Service and try again",
                    PREDATOR_SERVICE_DISPLAY_NAME,
                    name
                );
            }
            Ok(Self(handle))
        }

        fn query_status(&self) -> Result<SERVICE_STATUS_PROCESS> {
            let mut status = SERVICE_STATUS_PROCESS {
                dwServiceType: 0,
                dwCurrentState: 0,
                dwControlsAccepted: 0,
                dwWin32ExitCode: 0,
                dwServiceSpecificExitCode: 0,
                dwCheckPoint: 0,
                dwWaitHint: 0,
                dwProcessId: 0,
                dwServiceFlags: 0,
            };
            let mut bytes_needed = 0u32;
            let ok = unsafe {
                QueryServiceStatusEx(
                    self.0,
                    SC_STATUS_PROCESS_INFO,
                    (&mut status as *mut SERVICE_STATUS_PROCESS).cast(),
                    std::mem::size_of::<SERVICE_STATUS_PROCESS>() as u32,
                    &mut bytes_needed,
                )
            };
            if ok == 0 {
                bail!(
                    "QueryServiceStatusEx failed while checking {}: Windows error {}",
                    PREDATOR_SERVICE_NAME,
                    unsafe { GetLastError() }
                );
            }
            Ok(status)
        }

        fn query_startup_type(&self) -> Result<u32> {
            let mut bytes_needed = 0u32;
            unsafe {
                windows_sys::Win32::System::Services::QueryServiceConfigW(
                    self.0,
                    ptr::null_mut(),
                    0,
                    &mut bytes_needed,
                );
            }
            if bytes_needed == 0 {
                bail!("QueryServiceConfigW size probe failed");
            }

            let mut buffer = vec![0u8; bytes_needed as usize];
            let ok = unsafe {
                windows_sys::Win32::System::Services::QueryServiceConfigW(
                    self.0,
                    buffer.as_mut_ptr().cast(),
                    bytes_needed,
                    &mut bytes_needed,
                )
            };
            if ok == 0 {
                bail!(
                    "QueryServiceConfigW failed while checking {}: Windows error {}",
                    PREDATOR_SERVICE_NAME,
                    unsafe { GetLastError() }
                );
            }
            let config = unsafe {
                &*(buffer.as_ptr()
                    as *const windows_sys::Win32::System::Services::QUERY_SERVICE_CONFIGW)
            };
            Ok(config.dwStartType)
        }
    }

    impl Drop for ServiceHandle {
        fn drop(&mut self) {
            unsafe {
                CloseServiceHandle(self.0);
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::Result;
    use anyhow::bail;

    pub fn ensure_predator_service_running() -> Result<()> {
        bail!("Predator service checks are only available on Windows");
    }
}
