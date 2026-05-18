use crate::error::Result;
use crate::platform::pipe::ADMIN_PIPE_PREFIX;

pub fn admin_pipe_name(session_id: u32) -> String {
    format!("{ADMIN_PIPE_PREFIX}{session_id}")
}

pub fn current_admin_pipe_name() -> Result<String> {
    Ok(admin_pipe_name(current_session_id()?))
}

pub fn global_candidate_session_ids() -> Vec<u32> {
    let mut ids = Vec::new();
    if let Ok(current) = current_session_id() {
        ids.push(current);
    }
    ids.extend([1, 2]);

    if let Ok(entries) = std::fs::read_dir(r"\\.\pipe\") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(id) = name
                .strip_prefix("PredatorSense_admin_agent_")
                .and_then(|raw| raw.parse::<u32>().ok())
            {
                ids.push(id);
            }
        }
    }

    let mut unique = Vec::new();
    for id in ids {
        if !unique.contains(&id) {
            unique.push(id);
        }
    }
    unique
}

pub fn current_session_id() -> Result<u32> {
    platform::current_session_id()
}

#[cfg(windows)]
mod platform {
    use crate::error::Result;
    use anyhow::bail;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;

    pub fn current_session_id() -> Result<u32> {
        let mut session_id = 0u32;
        let ok = unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id) };
        if ok == 0 {
            bail!("ProcessIdToSessionId failed: Windows error {}", unsafe {
                GetLastError()
            });
        }
        Ok(session_id)
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::Result;
    use anyhow::bail;

    pub fn current_session_id() -> Result<u32> {
        bail!("session IDs are only available on Windows");
    }
}
