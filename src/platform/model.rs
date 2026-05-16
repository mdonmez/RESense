use crate::error::Result;
use crate::platform::registry;
use anyhow::bail;

const BIOS_PATH: &str = r"HARDWARE\DESCRIPTION\System\BIOS";
const SYSTEM_PRODUCT_NAME: &str = "SystemProductName";
const BASEBOARD_PRODUCT: &str = "BaseBoardProduct";
const SUPPORTED_MODEL_TOKENS: &[&str] = &["an515-58", "nitro an515-58", "jimny_adh"];

#[derive(Debug, Clone)]
pub struct ModelProbe {
    pub system_product_name: Option<String>,
    pub baseboard_product: Option<String>,
    pub nitrosense_model_name_1st: Option<String>,
}

pub fn ensure_supported_model(allow_any_model: bool) -> Result<()> {
    if allow_any_model {
        return Ok(());
    }

    let probe = read_model_probe();
    if is_supported_model(&probe) {
        return Ok(());
    }

    bail!(
        "unsupported model: RESense is intended for Acer Nitro AN515-58. Detected system_product_name={:?}, baseboard_product={:?}, nitrosense_model_name_1st={:?}. Use --dangerously-allow-any-model to bypass this check",
        probe.system_product_name,
        probe.baseboard_product,
        probe.nitrosense_model_name_1st
    );
}

fn read_model_probe() -> ModelProbe {
    ModelProbe {
        system_product_name: registry::read_hklm_string(BIOS_PATH, SYSTEM_PRODUCT_NAME).ok(),
        baseboard_product: registry::read_hklm_string(BIOS_PATH, BASEBOARD_PRODUCT).ok(),
        nitrosense_model_name_1st: registry::read_hklm_string(
            registry::NITROSENSE,
            "Model_Name_1st",
        )
        .ok(),
    }
}

fn is_supported_model(probe: &ModelProbe) -> bool {
    [
        probe.system_product_name.as_deref(),
        probe.baseboard_product.as_deref(),
        probe.nitrosense_model_name_1st.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(normalize_model)
    .any(|value| {
        SUPPORTED_MODEL_TOKENS
            .iter()
            .any(|token| value.contains(&normalize_model(token)))
    })
}

fn normalize_model(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_model_variants() {
        let probe = ModelProbe {
            system_product_name: Some("Nitro AN515-58".to_string()),
            baseboard_product: None,
            nitrosense_model_name_1st: None,
        };
        assert!(is_supported_model(&probe));

        let probe = ModelProbe {
            system_product_name: None,
            baseboard_product: Some("Jimny_ADH".to_string()),
            nitrosense_model_name_1st: None,
        };
        assert!(is_supported_model(&probe));
    }

    #[test]
    fn rejects_other_models() {
        let probe = ModelProbe {
            system_product_name: Some("Nitro AN515-45".to_string()),
            baseboard_product: Some("SomethingElse".to_string()),
            nitrosense_model_name_1st: None,
        };
        assert!(!is_supported_model(&probe));
    }
}
