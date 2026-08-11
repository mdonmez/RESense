use crate::cli::{FanCustomArgs, KeyboardDynamicArgs, KeyboardDynamicMode, KeyboardStaticArgs};
use anyhow::{Result as AnyhowResult, bail};

pub type Result<T> = AnyhowResult<T>;

pub mod validate {
    use super::*;

    pub fn range(name: &str, value: u8, min: u8, max: u8) -> Result<()> {
        if value < min || value > max {
            bail!("{name} must be between {min} and {max}");
        }
        Ok(())
    }

    pub fn fan_custom_args(args: &FanCustomArgs) -> Result<()> {
        if args.cpu.is_none() && args.gpu.is_none() && !args.cpu_auto && !args.gpu_auto {
            bail!("provide at least one of --cpu, --gpu, --cpu-auto, or --gpu-auto");
        }
        if args.cpu.is_some() && args.cpu_auto {
            bail!("choose either --cpu or --cpu-auto for the CPU fan");
        }
        if args.gpu.is_some() && args.gpu_auto {
            bail!("choose either --gpu or --gpu-auto for the GPU fan");
        }
        if let Some(percent) = args.cpu {
            range("cpu", percent, 0, 100)?;
        }
        if let Some(percent) = args.gpu {
            range("gpu", percent, 0, 100)?;
        }
        Ok(())
    }

    pub fn static_args(args: &KeyboardStaticArgs) -> Result<()> {
        if args.zone1.is_none()
            && args.zone2.is_none()
            && args.zone3.is_none()
            && args.zone4.is_none()
        {
            bail!("provide at least one of --zone1, --zone2, --zone3, or --zone4");
        }
        Ok(())
    }

    pub fn dynamic_args(args: &KeyboardDynamicArgs) -> Result<()> {
        if let Some(speed) = args.speed {
            range("speed", speed, 1, 9)?;
        }

        let uses_color = !matches!(args.mode, KeyboardDynamicMode::Neon);
        let uses_direction = matches!(
            args.mode,
            KeyboardDynamicMode::Wave | KeyboardDynamicMode::Shifting
        );

        if !uses_color && args.color.is_some() {
            bail!("{:?} mode does not use --color", args.mode);
        }
        if !uses_direction && args.direction.is_some() {
            bail!("{:?} mode does not use --direction", args.mode);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::cli::{KeyboardDynamicArgs, KeyboardDynamicMode};

    #[test]
    fn dynamic_speed_accepts_nine() {
        let args = KeyboardDynamicArgs {
            mode: KeyboardDynamicMode::Wave,
            speed: Some(9),
            color: None,
            direction: None,
        };

        assert!(validate::dynamic_args(&args).is_ok());
    }

    #[test]
    fn dynamic_speed_rejects_ten() {
        let args = KeyboardDynamicArgs {
            mode: KeyboardDynamicMode::Wave,
            speed: Some(10),
            color: None,
            direction: None,
        };

        assert!(validate::dynamic_args(&args).is_err());
    }
}
