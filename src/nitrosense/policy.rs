use crate::cli::{FanMode, OperatingMode};
use crate::error::Result;
use anyhow::bail;

pub fn ensure_fan_control_allowed() -> Result<()> {
    let mode = super::mode::read_state()?;
    if !is_quiet_mode_name(&mode.mode) {
        return Ok(());
    }

    bail!("RESense disables fan control while quiet mode is active");
}

pub fn enforce_operation_mode_fan_policy(mode: OperatingMode) -> Result<()> {
    if matches!(mode, OperatingMode::Quiet) {
        force_fans_auto()?;
    }
    Ok(())
}

fn force_fans_auto() -> Result<()> {
    super::fan::set_mode(FanMode::Auto)
}

fn is_quiet_mode_name(mode: &str) -> bool {
    mode == "quiet"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_mode_is_the_only_fan_lock_condition() {
        assert!(matches!(
            ensure_fan_control_allowed_for_mode("quiet"),
            Err(_)
        ));
        assert!(ensure_fan_control_allowed_for_mode("default").is_ok());
        assert!(ensure_fan_control_allowed_for_mode("performance").is_ok());
    }

    fn ensure_fan_control_allowed_for_mode(mode: &str) -> Result<()> {
        if !is_quiet_mode_name(mode) {
            return Ok(());
        }
        bail!("RESense disables fan control while quiet mode is active");
    }

    #[test]
    fn recognizes_quiet_mode_name() {
        assert!(is_quiet_mode_name("quiet"));
        assert!(!is_quiet_mode_name("default"));
    }
}
