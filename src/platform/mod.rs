mod lighting_store;
mod registry;
mod windows;

pub(crate) mod pipe;

use crate::error::Result;
use anyhow::bail;
use pipe::{Argument, PipeClient};

const BIOS_SYSTEM_PRODUCT_NAME: &str = "SystemProductName";
const BIOS_BASEBOARD_PRODUCT: &str = "BaseBoardProduct";
const SUPPORTED_MODEL_TOKENS: &[&str] = &["an515-58", "nitro an515-58", "jimny_adh"];

pub(crate) struct Platform {
    service: PipeClient,
    current_session_id: u32,
}

impl Platform {
    pub(crate) fn connect(allow_any_model: bool) -> Result<Self> {
        if !allow_any_model {
            ensure_supported_model()?;
        }
        let current_session = windows::current_session_id()?;
        Ok(Self {
            service: PipeClient::service(),
            current_session_id: current_session,
        })
    }

    pub(crate) fn service_set(&self, command: u16, args: &[Argument<'_>]) -> Result<u32> {
        Ok(self.service.set(command, args)?.return_code)
    }

    pub(crate) fn service_get_u64(&self, command: u16, args: &[Argument<'_>]) -> Result<u64> {
        self.service.get_u64(command, args)
    }

    pub(crate) fn current_admin_fire(&self, command: u16, args: &[Argument<'_>]) -> Result<()> {
        PipeClient::admin(self.current_session_id).fire(command, args)
    }

    pub(crate) fn shared_admin_fire(&self, command: u16, args: &[Argument<'_>]) -> Result<()> {
        self.try_shared(|pipe| pipe.fire(command, args))
    }

    pub(crate) fn shared_admin_get_u32(&self, command: u16, args: &[Argument<'_>]) -> Result<u32> {
        self.try_shared(|pipe| pipe.get_u32(command, args))
    }

    pub(crate) fn read_dword(&self, path: &str, name: &str) -> Result<u32> {
        registry::read_dword(path, name)
    }

    pub(crate) fn read_optional_string(&self, path: &str, name: &str) -> Result<Option<String>> {
        registry::read_optional_string(path, name)
    }

    pub(crate) fn set_dwords(&self, updates: &[(&str, &str, u32)]) -> Result<()> {
        registry::set_dwords(&self.service, updates)
    }

    pub(crate) fn read_sticky_keys(&self) -> Result<bool> {
        windows::read_sticky_keys()
    }

    pub(crate) fn atomic_replace(
        &self,
        temporary: &std::path::Path,
        target: &std::path::Path,
    ) -> Result<()> {
        windows::replace_file_atomic(temporary, target)
    }

    fn try_shared<T>(&self, operation: impl Fn(&PipeClient) -> Result<T>) -> Result<T> {
        let mut last_error = None;
        for session_id in windows::admin_session_ids(self.current_session_id) {
            match operation(&PipeClient::admin(session_id)) {
                Ok(value) => return Ok(value),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("no PredatorSense admin session is available")))
    }
}

#[cfg(feature = "dev-tools")]
pub(crate) fn current_session_id() -> Result<u32> {
    windows::current_session_id()
}

fn ensure_supported_model() -> Result<()> {
    let values = [
        registry::read_optional_string(registry::BIOS, BIOS_SYSTEM_PRODUCT_NAME)?,
        registry::read_optional_string(registry::BIOS, BIOS_BASEBOARD_PRODUCT)?,
        registry::read_optional_string(registry::NITROSENSE, "Model_Name_1st")?,
    ];
    let supported = values
        .iter()
        .filter_map(Option::as_deref)
        .map(normalize_model)
        .any(|value| {
            SUPPORTED_MODEL_TOKENS
                .iter()
                .map(|token| normalize_model(token))
                .any(|token| value.contains(&token))
        });
    if supported {
        return Ok(());
    }
    bail!(
        "unsupported model: RESense is intended for Acer Nitro AN515-58; detected system_product_name={:?}, baseboard_product={:?}, nitrosense_model_name_1st={:?}. Use --dangerously-allow-any-model to bypass this check",
        values[0],
        values[1],
        values[2]
    )
}

fn normalize_model(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

pub(crate) use lighting_store::LightingStore;
pub(crate) use registry::{ADVANCED_SETTINGS, FAN_CONTROL, LIGHT_SETTING, NITROSENSE, OVERCLOCK};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_model_names() {
        assert_eq!(normalize_model("Nitro AN515-58"), "nitroan515-58");
    }
}
