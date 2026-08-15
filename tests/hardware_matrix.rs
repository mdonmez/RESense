#[cfg(windows)]
mod hardware {
    use anyhow::{Context, Result, anyhow, bail};
    use resense::device::{
        Brightness, Device, Direction, DynamicEffect, DynamicLighting, DynamicMode, DynamicRequest,
        DynamicSpeed, FanChange, FanCustomControl, FanCustomRequest, FanMode, FanState,
        LightingMode, OperationMode, Percent, Rgb, SoundPreset, StaticRequest, SystemState,
        ZoneChange,
    };
    use std::thread;
    use std::time::Duration;

    const OPERATION_SETTLE_DELAY: Duration = Duration::from_secs(1);

    #[derive(Clone, Copy)]
    enum FanPlan {
        Auto,
        Max,
        Custom(FanCustomRequest),
    }

    #[derive(Clone, Copy)]
    struct StaticPlan {
        request: StaticRequest,
        enabled: [bool; 4],
        colors: [Option<Rgb>; 4],
    }

    #[derive(Clone, Copy)]
    struct DynamicPlan {
        request: DynamicRequest,
        expected: DynamicLighting,
    }

    #[derive(Clone, Copy)]
    enum LightingPlan {
        Static(StaticPlan),
        Dynamic(DynamicPlan),
    }

    #[derive(Clone, Copy)]
    struct Cycle {
        fan: FanPlan,
        lighting: LightingPlan,
        brightness: Brightness,
        timeout: bool,
        sticky_keys: bool,
        win_menu_lock: bool,
        display_overdrive: bool,
        sound: SoundPreset,
        mode: OperationMode,
    }

    pub fn run() -> Result<()> {
        let device = Device::connect(false).context("connecting to RESense")?;
        let original = device
            .status()
            .context("capturing the original hardware state")?;
        println!("Captured the original configuration state.");

        let restore = RestoreGuard::new(&device, original);
        let exercise_result = exercise(&device, &original);
        let restore_result = restore.restore();
        combine_results(exercise_result, restore_result)
    }

    fn exercise(device: &Device, original: &SystemState) -> Result<()> {
        if device.mode()? == OperationMode::Quiet {
            println!("Leaving quiet mode temporarily so fan writes are available.");
            let observed = device.set_mode(OperationMode::Default, false)?;
            settle_after_operation();
            expect_mode(observed, OperationMode::Default)?;
        }

        let cycles = build_cycles()?;
        let display_supported = original.display_overdrive.is_some();
        let sound_supported = original.sound.is_some();
        if !display_supported {
            println!("Skipping LCD Overdrive cycles: the hardware reports it unsupported.");
        }
        if !sound_supported {
            println!("Skipping sound cycles: the active output reports the DTS path unsupported.");
        }

        for (index, cycle) in cycles.iter().enumerate() {
            println!("Running hardware cycle {}/3.", index + 1);
            apply_fan(device, cycle.fan)?;
            apply_keyboard(device, cycle.lighting, cycle)?;

            if display_supported {
                let observed = device.set_display_overdrive(cycle.display_overdrive)?;
                settle_after_operation();
                if observed != Some(cycle.display_overdrive) {
                    bail!(
                        "LCD Overdrive cycle {} readback mismatch: expected {}, got {observed:?}",
                        index + 1,
                        cycle.display_overdrive
                    );
                }
            }

            if sound_supported {
                let observed = device.set_sound(cycle.sound)?;
                settle_after_operation();
                if observed != cycle.sound {
                    bail!(
                        "sound cycle {} readback mismatch: expected {:?}, got {observed:?}",
                        index + 1,
                        cycle.sound
                    );
                }
            }

            let observed = device.set_mode(cycle.mode, false)?;
            settle_after_operation();
            expect_mode(observed, cycle.mode)?;
        }
        Ok(())
    }

    fn apply_fan(device: &Device, plan: FanPlan) -> Result<()> {
        let observed = match plan {
            FanPlan::Auto => device.set_fan_auto()?,
            FanPlan::Max => device.set_fan_max()?,
            FanPlan::Custom(request) => device.set_fan_custom(request)?,
        };
        settle_after_operation();
        verify_fan(&observed, plan)
    }

    fn verify_fan(state: &FanState, plan: FanPlan) -> Result<()> {
        match plan {
            FanPlan::Auto => expect_fan_mode(state, FanMode::Auto),
            FanPlan::Max => expect_fan_mode(state, FanMode::Max),
            FanPlan::Custom(request) => {
                expect_fan_mode(state, FanMode::Custom)?;
                verify_custom_control(state.custom.cpu, request.cpu(), "CPU")?;
                verify_custom_control(state.custom.gpu, request.gpu(), "GPU")
            }
        }
    }

    fn expect_fan_mode(state: &FanState, expected: FanMode) -> Result<()> {
        if state.mode != expected {
            bail!(
                "fan readback mismatch: expected {expected:?}, got {:?}",
                state.mode
            );
        }
        Ok(())
    }

    fn verify_custom_control(
        observed: FanCustomControl,
        expected: Option<FanChange>,
        name: &str,
    ) -> Result<()> {
        match expected {
            Some(FanChange::Auto) if matches!(observed, FanCustomControl::Auto { .. }) => Ok(()),
            Some(FanChange::Manual(percent))
                if observed == (FanCustomControl::Manual { percent }) =>
            {
                Ok(())
            }
            Some(expected) => {
                bail!("{name} fan readback mismatch: expected {expected:?}, got {observed:?}")
            }
            None => Ok(()),
        }
    }

    fn apply_keyboard(device: &Device, lighting: LightingPlan, cycle: &Cycle) -> Result<()> {
        let before = device.keyboard()?;
        match lighting {
            LightingPlan::Static(plan) => {
                device.set_keyboard_static(plan.request)?;
                settle_after_operation();
            }
            LightingPlan::Dynamic(plan) => {
                device.set_keyboard_dynamic(plan.request)?;
                settle_after_operation();
            }
        }
        device.set_keyboard_brightness(cycle.brightness)?;
        settle_after_operation();
        device.set_keyboard_timeout(cycle.timeout)?;
        settle_after_operation();
        device.set_sticky_keys(cycle.sticky_keys)?;
        settle_after_operation();
        device.set_win_menu_lock(cycle.win_menu_lock)?;
        settle_after_operation();

        let observed = device.keyboard()?;
        verify_keyboard(&before, &observed, lighting, cycle)
    }

    fn verify_keyboard(
        before: &resense::device::KeyboardState,
        state: &resense::device::KeyboardState,
        lighting: LightingPlan,
        cycle: &Cycle,
    ) -> Result<()> {
        if state.brightness != cycle.brightness
            || state.backlight_timeout != cycle.timeout
            || state.sticky_keys != cycle.sticky_keys
            || state.win_menu_lock != cycle.win_menu_lock
        {
            bail!("keyboard readback mismatch: got {state:?}");
        }

        match lighting {
            LightingPlan::Static(plan) => {
                if state.lighting.mode != LightingMode::Static
                    || state.lighting.dynamic != before.lighting.dynamic
                {
                    bail!("expected static keyboard lighting with preserved dynamic settings");
                }
                for (index, zone) in state.lighting.static_zones.iter().enumerate() {
                    if zone.enabled != plan.enabled[index]
                        || plan.colors[index].is_some_and(|color| zone.color != color)
                    {
                        bail!("static keyboard zone {} readback mismatch", index + 1);
                    }
                }
                Ok(())
            }
            LightingPlan::Dynamic(plan) => {
                if state.lighting.mode != LightingMode::Dynamic
                    || state.lighting.static_zones != before.lighting.static_zones
                    || state.lighting.dynamic != plan.expected
                {
                    bail!("dynamic keyboard lighting readback mismatch");
                }
                Ok(())
            }
        }
    }

    fn build_cycles() -> Result<[Cycle; 3]> {
        let red = Rgb::parse("ED1020")?;
        let green = Rgb::parse("00FF66")?;
        let blue = Rgb::parse("1464FF")?;
        let cyan = Rgb::parse("00FFFF")?;
        let orange = Rgb::parse("FF8000")?;

        let static_one = StaticPlan {
            request: StaticRequest::new([
                Some(ZoneChange::Color(red)),
                Some(ZoneChange::Color(green)),
                Some(ZoneChange::Color(blue)),
                Some(ZoneChange::Color(orange)),
            ])?,
            enabled: [true; 4],
            colors: [Some(red), Some(green), Some(blue), Some(orange)],
        };
        let dynamic_two = DynamicPlan {
            request: DynamicRequest::new(
                DynamicMode::Wave,
                Some(DynamicSpeed::new(2)?),
                Some(cyan),
                Some(Direction::FromLeft),
            )?,
            expected: DynamicLighting {
                effect: DynamicEffect::Wave {
                    color: cyan,
                    direction: Direction::FromLeft,
                },
                speed: DynamicSpeed::new(2)?,
            },
        };
        let static_three = StaticPlan {
            request: StaticRequest::new([
                Some(ZoneChange::Color(orange)),
                Some(ZoneChange::Off),
                Some(ZoneChange::Color(cyan)),
                Some(ZoneChange::Off),
            ])?,
            enabled: [true, false, true, false],
            colors: [Some(orange), None, Some(cyan), None],
        };

        Ok([
            Cycle {
                fan: FanPlan::Auto,
                lighting: LightingPlan::Static(static_one),
                brightness: Brightness::new(1)?,
                timeout: true,
                sticky_keys: true,
                win_menu_lock: false,
                display_overdrive: true,
                sound: SoundPreset::Music,
                mode: OperationMode::Performance,
            },
            Cycle {
                fan: FanPlan::Max,
                lighting: LightingPlan::Dynamic(dynamic_two),
                brightness: Brightness::new(3)?,
                timeout: false,
                sticky_keys: false,
                win_menu_lock: true,
                display_overdrive: false,
                sound: SoundPreset::Movies,
                mode: OperationMode::Default,
            },
            Cycle {
                fan: FanPlan::Custom(FanCustomRequest::new(
                    Some(FanChange::Manual(Percent::new(65)?)),
                    Some(FanChange::Manual(Percent::new(55)?)),
                )?),
                lighting: LightingPlan::Static(static_three),
                brightness: Brightness::new(5)?,
                timeout: true,
                sticky_keys: true,
                win_menu_lock: false,
                display_overdrive: true,
                sound: SoundPreset::Voice,
                mode: OperationMode::Quiet,
            },
        ])
    }

    fn restore_state(device: &Device, original: &SystemState) -> Result<()> {
        if device.mode()? == OperationMode::Quiet {
            device.set_mode(OperationMode::Default, false)?;
            settle_after_operation();
        }
        restore_fans(device, original.fan)?;
        restore_keyboard(device, original.keyboard)?;
        if let Some(enabled) = original.display_overdrive {
            device.set_display_overdrive(enabled)?;
            settle_after_operation();
        }
        if let Some(preset) = original.sound {
            device.set_sound(preset)?;
            settle_after_operation();
        }
        device.set_mode(original.mode, false)?;
        settle_after_operation();

        let observed = device.status().context("reading state after restoration")?;
        if !persistent_state_matches(original, &observed) {
            bail!(
                "restored configuration differs from the captured configuration:\nexpected={original:?}\nobserved={observed:?}"
            );
        }
        Ok(())
    }

    fn restore_fans(device: &Device, original: FanState) -> Result<()> {
        let remembered = FanCustomRequest::new(
            Some(FanChange::Manual(original.custom.cpu.percent())),
            Some(FanChange::Manual(original.custom.gpu.percent())),
        )?;
        device.set_fan_custom(remembered)?;
        settle_after_operation();

        let exact = FanCustomRequest::new(
            Some(change_from_control(original.custom.cpu)),
            Some(change_from_control(original.custom.gpu)),
        )?;
        device.set_fan_custom(exact)?;
        settle_after_operation();
        match original.mode {
            FanMode::Auto => {
                device.set_fan_auto()?;
                settle_after_operation();
            }
            FanMode::Max => {
                device.set_fan_max()?;
                settle_after_operation();
            }
            FanMode::Custom => {}
        }
        Ok(())
    }

    fn change_from_control(control: FanCustomControl) -> FanChange {
        match control {
            FanCustomControl::Auto { .. } => FanChange::Auto,
            FanCustomControl::Manual { percent } => FanChange::Manual(percent),
        }
    }

    fn restore_keyboard(device: &Device, original: resense::device::KeyboardState) -> Result<()> {
        let zones = original.lighting.static_zones;
        let all_enabled =
            StaticRequest::new(zones.map(|zone| Some(ZoneChange::Color(zone.color))))?;
        device.set_keyboard_static(all_enabled)?;
        settle_after_operation();
        let exact_zones = StaticRequest::new(zones.map(|zone| {
            Some(if zone.enabled {
                ZoneChange::Color(zone.color)
            } else {
                ZoneChange::Off
            })
        }))?;
        device.set_keyboard_static(exact_zones)?;
        settle_after_operation();

        device.set_keyboard_dynamic(dynamic_request(original.lighting.dynamic)?)?;
        settle_after_operation();
        if original.lighting.mode == LightingMode::Static {
            device.set_keyboard_static(exact_zones)?;
            settle_after_operation();
        }
        device.set_keyboard_brightness(original.brightness)?;
        settle_after_operation();
        device.set_keyboard_timeout(original.backlight_timeout)?;
        settle_after_operation();
        device.set_sticky_keys(original.sticky_keys)?;
        settle_after_operation();
        device.set_win_menu_lock(original.win_menu_lock)?;
        settle_after_operation();
        Ok(())
    }

    fn settle_after_operation() {
        thread::sleep(OPERATION_SETTLE_DELAY);
    }

    fn dynamic_request(effect: DynamicLighting) -> Result<DynamicRequest> {
        match effect.effect {
            DynamicEffect::Breathing { color } => DynamicRequest::new(
                DynamicMode::Breathing,
                Some(effect.speed),
                Some(color),
                None,
            ),
            DynamicEffect::Neon => {
                DynamicRequest::new(DynamicMode::Neon, Some(effect.speed), None, None)
            }
            DynamicEffect::Shifting { color, direction } => DynamicRequest::new(
                DynamicMode::Shifting,
                Some(effect.speed),
                Some(color),
                Some(direction),
            ),
            DynamicEffect::Wave { color, direction } => DynamicRequest::new(
                DynamicMode::Wave,
                Some(effect.speed),
                Some(color),
                Some(direction),
            ),
            DynamicEffect::Zoom { color } => {
                DynamicRequest::new(DynamicMode::Zoom, Some(effect.speed), Some(color), None)
            }
        }
    }

    fn persistent_state_matches(expected: &SystemState, observed: &SystemState) -> bool {
        expected.fan.mode == observed.fan.mode
            && expected.fan.custom == observed.fan.custom
            && expected.keyboard == observed.keyboard
            && expected.mode == observed.mode
            && expected.display_overdrive == observed.display_overdrive
            && expected.sound == observed.sound
    }

    fn expect_mode(observed: OperationMode, expected: OperationMode) -> Result<()> {
        if observed != expected {
            bail!("operation mode readback mismatch: expected {expected:?}, got {observed:?}");
        }
        Ok(())
    }

    fn combine_results(test: Result<()>, restore: Result<()>) -> Result<()> {
        match (test, restore) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(test), Ok(())) => Err(test).context("hardware cycle failed; state was restored"),
            (Ok(()), Err(restore)) => Err(restore).context("hardware state restoration failed"),
            (Err(test), Err(restore)) => Err(anyhow!(
                "hardware cycle failed: {test:#}; state restoration also failed: {restore:#}"
            )),
        }
    }

    struct RestoreGuard<'a> {
        device: &'a Device,
        original: SystemState,
        restored: bool,
    }

    impl<'a> RestoreGuard<'a> {
        fn new(device: &'a Device, original: SystemState) -> Self {
            Self {
                device,
                original,
                restored: false,
            }
        }

        fn restore(mut self) -> Result<()> {
            let result = restore_state(self.device, &self.original);
            if result.is_ok() {
                self.restored = true;
            }
            result
        }
    }

    impl Drop for RestoreGuard<'_> {
        fn drop(&mut self) {
            if !self.restored
                && let Err(error) = restore_state(self.device, &self.original)
            {
                eprintln!("automatic hardware-state restoration failed: {error:#}");
            }
        }
    }
}

#[cfg(windows)]
#[test]
#[ignore = "writes real hardware; run explicitly with --features hardware-tests -- --ignored"]
fn hardware_matrix() {
    hardware::run().expect("hardware matrix failed")
}

#[cfg(not(windows))]
#[test]
#[ignore = "RESense hardware tests require Windows"]
fn hardware_matrix_requires_windows() {
    panic!("RESense hardware tests require Windows");
}
