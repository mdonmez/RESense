use crate::error::Result;
use crate::platform::{LIGHT_SETTING, Platform};
use anyhow::{Context, bail};
use std::fs::File;
use std::path::PathBuf;
use xmltree::Element;

const PROFILE_ROOT: &str = r"C:\ProgramData\OEM\NitroSense\ProfilePool\LightProfilePool";
const HW_SUPPORT_PATH: &str = r"C:\ProgramData\OEM\NitroSense\HW_Support.ini";

pub(crate) struct LightingStore {
    profile_path: PathBuf,
    hw_support_path: PathBuf,
}

impl LightingStore {
    pub(crate) fn resolve(platform: &Platform) -> Result<Self> {
        let profile = match platform.read_optional_string(LIGHT_SETTING, "LightingProfile")? {
            Some(value) if !value.trim().is_empty() => value,
            _ => "Default".to_string(),
        };
        Ok(Self {
            profile_path: PathBuf::from(PROFILE_ROOT).join(profile).join("Main.xml"),
            hw_support_path: PathBuf::from(HW_SUPPORT_PATH),
        })
    }

    pub(crate) fn read(&self) -> Result<Element> {
        Element::parse(
            File::open(&self.profile_path)
                .with_context(|| format!("opening {}", self.profile_path.display()))?,
        )
        .with_context(|| format!("parsing {}", self.profile_path.display()))
    }

    pub(crate) fn write(&self, root: &Element, platform: &Platform) -> Result<()> {
        let parent = self
            .profile_path
            .parent()
            .context("keyboard profile path has no parent directory")?;
        let temporary = parent.join(format!(".Main.xml.resense-{}.tmp", std::process::id()));
        let write_result: Result<()> = (|| {
            let mut file = File::create(&temporary)
                .with_context(|| format!("creating {}", temporary.display()))?;
            root.write(&mut file)
                .with_context(|| format!("writing {}", temporary.display()))?;
            file.sync_all()
                .with_context(|| format!("flushing {}", temporary.display()))?;
            let mut validation = File::open(&temporary)
                .with_context(|| format!("reopening {} for validation", temporary.display()))?;
            Element::parse(&mut validation)
                .with_context(|| format!("validating {}", temporary.display()))?;
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        if let Err(error) = platform.atomic_replace(&temporary, &self.profile_path) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error).with_context(|| {
                format!(
                    "atomically replacing {} with {}",
                    self.profile_path.display(),
                    temporary.display()
                )
            });
        }
        Ok(())
    }

    pub(crate) fn color_adjustment(&self) -> Result<(f32, f32, f32)> {
        let text = std::fs::read_to_string(&self.hw_support_path)
            .with_context(|| format!("reading {}", self.hw_support_path.display()))?;
        let mut in_section = false;
        let mut values = [None; 3];
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') && line.ends_with(']') {
                in_section = line == "[ZoneColorAdjust]";
                continue;
            }
            if !in_section {
                continue;
            }
            let Some((key, raw_value)) = line.split_once('=') else {
                continue;
            };
            let value: f32 = raw_value
                .trim()
                .parse()
                .with_context(|| format!("invalid ZoneColorAdjust value {raw_value:?}"))?;
            if !value.is_finite() || value < 0.0 {
                bail!("invalid ZoneColorAdjust multiplier {value}");
            }
            match key.trim() {
                "R" => values[0] = Some(value),
                "G" => values[1] = Some(value),
                "B" => values[2] = Some(value),
                _ => {}
            }
        }
        Ok((
            values[0].context("ZoneColorAdjust.R is missing")?,
            values[1].context("ZoneColorAdjust.G is missing")?,
            values[2].context("ZoneColorAdjust.B is missing")?,
        ))
    }
}
