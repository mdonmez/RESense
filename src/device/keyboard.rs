use super::{
    Brightness, Direction, DynamicEffect, DynamicLighting, DynamicMode, DynamicRequest,
    DynamicSpeed, KeyboardLightingState, KeyboardState, LightingMode, Rgb, StaticRequest, Zone,
    ZoneChange,
};
use crate::error::Result;
use crate::platform::pipe::Argument;
use crate::platform::{ADVANCED_SETTINGS, LightingStore, NITROSENSE, Platform};
use anyhow::{Context, bail};
use std::thread;
use std::time::Duration;
use xmltree::{Element, XMLNode};

const CMD_SET_KB_BACKLIGHT: u16 = 27;
const CMD_SET_RGB_KB: u16 = 28;
const CMD_SET_LED_BEHAVIOR: u16 = 29;
const CMD_ADMIN_SET_STICKY_KEYS: u16 = 2;
const CMD_SET_GAMING_PROFILE: u16 = 9;
const CMD_GET_GAMING_PROFILE: u16 = 10;
const QUERY_GAMING_PROFILE: u32 = 0;
const WIN_MENU_SELECTOR: u64 = 2;
const WIN_MENU_STATUS_SHIFT: u64 = 24;
const COLOR_TAG_COUNT: usize = 127;
const CMD_WMI_SET_FUNCTION: u16 = 17;
const CMD_WMI_GET_FUNCTION: u16 = 20;
const TIMEOUT_SECONDS: u8 = 30;
const STICKY_KEYS_SETTLE_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct LightingSnapshot {
    brightness: Brightness,
    lighting: KeyboardLightingState,
}

pub(crate) fn read(platform: &Platform) -> Result<KeyboardState> {
    let lighting = read_lighting(platform)?;
    Ok(KeyboardState {
        brightness: lighting.brightness,
        lighting: lighting.lighting,
        backlight_timeout: read_timeout(platform)?,
        sticky_keys: platform.read_sticky_keys()?,
        win_menu_lock: read_win_menu_lock(platform)?,
    })
}

pub(crate) fn set_brightness(platform: &Platform, brightness: Brightness) -> Result<KeyboardState> {
    let current = read_lighting(platform)?;
    let store = LightingStore::resolve(platform)?;
    let payload = match current.lighting.mode {
        LightingMode::Static => encode_brightness(brightness),
        LightingMode::Dynamic => {
            let request = request_from_dynamic(current.lighting.dynamic)?;
            encode_dynamic_payload(&request, brightness, &store)?
        }
    };
    ensure_success(
        CMD_SET_KB_BACKLIGHT,
        platform.service_set(CMD_SET_KB_BACKLIGHT, &[Argument::U64(payload)])?,
    )?;
    mutate_profile(platform, |root| {
        set_attr(child_mut(root, "Key")?, "brightness", brightness.get());
        set_attr(
            child_mut(root, "LightingEffects")?,
            "brightness",
            brightness.get(),
        );
        Ok(())
    })?;
    let state = read(platform)?;
    if state.lighting != current.lighting {
        bail!("keyboard lighting mode verification failed")
    }
    verify_brightness(state, brightness)
}

pub(crate) fn set_static(platform: &Platform, request: StaticRequest) -> Result<KeyboardState> {
    let current = read_lighting(platform)?;
    let zones = merge_zones(current.lighting.static_zones, request.zones);
    let brightness = current.brightness;
    ensure_success(
        CMD_SET_KB_BACKLIGHT,
        platform.service_set(
            CMD_SET_KB_BACKLIGHT,
            &[Argument::U64(encode_brightness(brightness))],
        )?,
    )?;
    ensure_success(
        CMD_SET_LED_BEHAVIOR,
        platform.service_set(
            CMD_SET_LED_BEHAVIOR,
            &[Argument::U64(encode_zone_behavior(&zones))],
        )?,
    )?;
    let store = LightingStore::resolve(platform)?;
    for (index, zone) in zones.iter().enumerate() {
        if zone.enabled {
            ensure_success(
                CMD_SET_RGB_KB,
                platform.service_set(
                    CMD_SET_RGB_KB,
                    &[Argument::U64(encode_zone_color(
                        (index + 1) as u8,
                        zone.color,
                        &store,
                    )?)],
                )?,
            )?;
        }
    }
    mutate_profile(platform, |root| update_static_xml(root, &zones, brightness))?;
    let state = read(platform)?;
    if state.lighting.mode != LightingMode::Static
        || state.lighting.static_zones != zones
        || state.lighting.dynamic != current.lighting.dynamic
        || state.brightness != brightness
    {
        bail!("static keyboard lighting verification failed")
    }
    Ok(state)
}

pub(crate) fn set_dynamic(platform: &Platform, request: DynamicRequest) -> Result<KeyboardState> {
    let current = read_lighting(platform)?;
    let store = LightingStore::resolve(platform)?;
    let payload = encode_dynamic_payload(&request, current.brightness, &store)?;
    ensure_success(
        CMD_SET_KB_BACKLIGHT,
        platform.service_set(CMD_SET_KB_BACKLIGHT, &[Argument::U64(payload)])?,
    )?;
    mutate_profile(platform, |root| {
        update_dynamic_xml(root, request, current.brightness)
    })?;
    let expected_effect = effect_from_request(request)?;
    let state = read(platform)?;
    if state.lighting.mode != LightingMode::Dynamic
        || state.lighting.static_zones != current.lighting.static_zones
        || state.lighting.dynamic.effect != expected_effect
        || state.lighting.dynamic.speed != request.speed
    {
        bail!("dynamic keyboard lighting verification failed")
    }
    Ok(state)
}

pub(crate) fn set_timeout(platform: &Platform, enabled: bool) -> Result<KeyboardState> {
    let hotkey = platform.read_dword(NITROSENSE, "BK_Hotkey_Number")?;
    let current = read_backlight_raw(platform, hotkey)?;
    let timeout = if enabled { TIMEOUT_SECONDS } else { 0 };
    let payload = encode_timeout_set(hotkey, current.brightness_percent, timeout);
    ensure_success(
        CMD_WMI_SET_FUNCTION,
        platform.service_set(CMD_WMI_SET_FUNCTION, &[Argument::U64(payload)])?,
    )?;
    let observed = read_timeout(platform)?;
    if observed != enabled {
        bail!("keyboard backlight timeout verification failed")
    }
    read(platform)
}

pub(crate) fn set_sticky_keys(platform: &Platform, enabled: bool) -> Result<KeyboardState> {
    platform.current_admin_fire(CMD_ADMIN_SET_STICKY_KEYS, &[Argument::U32(enabled as u32)])?;
    platform.set_dwords(&[(ADVANCED_SETTINGS, "StickyKey", enabled as u32)])?;
    thread::sleep(STICKY_KEYS_SETTLE_DELAY);
    if platform.read_sticky_keys()? != enabled {
        bail!("Sticky Keys verification failed")
    }
    read(platform)
}

pub(crate) fn set_win_menu_lock(platform: &Platform, enabled: bool) -> Result<KeyboardState> {
    let payload = WIN_MENU_SELECTOR | ((enabled as u64) << WIN_MENU_STATUS_SHIFT);
    ensure_success(
        CMD_SET_GAMING_PROFILE,
        platform.service_set(CMD_SET_GAMING_PROFILE, &[Argument::U64(payload)])?,
    )?;
    platform.set_dwords(&[(ADVANCED_SETTINGS, "WinAndMenuKey", enabled as u32)])?;
    if read_win_menu_lock(platform)? != enabled {
        bail!("Windows/Menu key lock verification failed")
    }
    read(platform)
}

fn read_lighting(platform: &Platform) -> Result<LightingSnapshot> {
    let store = LightingStore::resolve(platform)?;
    let root = store.read()?;
    parse_lighting(&root)
}

fn read_timeout(platform: &Platform) -> Result<bool> {
    let hotkey = platform.read_dword(NITROSENSE, "BK_Hotkey_Number")?;
    Ok(read_backlight_raw(platform, hotkey)?.timeout_seconds == TIMEOUT_SECONDS)
}

fn read_win_menu_lock(platform: &Platform) -> Result<bool> {
    let value = platform.service_get_u64(
        CMD_GET_GAMING_PROFILE,
        &[Argument::U32(QUERY_GAMING_PROFILE)],
    )?;
    Ok(((value >> WIN_MENU_STATUS_SHIFT) & 0xFF) == 1)
}

#[derive(Debug, Clone, Copy)]
struct BacklightRaw {
    brightness_percent: u8,
    timeout_seconds: u8,
}

fn read_backlight_raw(platform: &Platform, hotkey: u32) -> Result<BacklightRaw> {
    let value = platform.service_get_u64(
        CMD_WMI_GET_FUNCTION,
        &[Argument::U32(encode_timeout_get(hotkey))],
    )?;
    Ok(BacklightRaw {
        brightness_percent: ((value >> 32) & 0xFF) as u8,
        timeout_seconds: ((value >> 40) & 0xFF) as u8,
    })
}

fn encode_timeout_get(hotkey: u32) -> u32 {
    1 | (hotkey << 8) | 0x80000
}

fn encode_timeout_set(hotkey: u32, brightness_percent: u8, timeout: u8) -> u64 {
    (2 | (hotkey << 8) | 0x80000) as u64
        | ((brightness_percent as u64) << 32)
        | ((timeout as u64) << 40)
}

fn verify_brightness(state: KeyboardState, expected: Brightness) -> Result<KeyboardState> {
    if state.brightness != expected {
        bail!(
            "keyboard brightness verification failed: expected {}, got {}",
            expected.get(),
            state.brightness.get()
        )
    }
    Ok(state)
}

fn merge_zones(current: [Zone; 4], changes: [Option<ZoneChange>; 4]) -> [Zone; 4] {
    std::array::from_fn(|index| match changes[index] {
        None => current[index],
        Some(ZoneChange::Off) => Zone {
            enabled: false,
            color: current[index].color,
        },
        Some(ZoneChange::Color(color)) => Zone {
            enabled: true,
            color,
        },
    })
}

fn request_from_dynamic(dynamic: DynamicLighting) -> Result<DynamicRequest> {
    let (mode, color, direction) = match dynamic.effect {
        DynamicEffect::Breathing { color } => (DynamicMode::Breathing, Some(color), None),
        DynamicEffect::Neon => (DynamicMode::Neon, None, None),
        DynamicEffect::Shifting { color, direction } => {
            (DynamicMode::Shifting, Some(color), Some(direction))
        }
        DynamicEffect::Wave { direction } => (DynamicMode::Wave, None, Some(direction)),
        DynamicEffect::Zoom { color } => (DynamicMode::Zoom, Some(color), None),
    };
    DynamicRequest::new(mode, dynamic.speed, color, direction)
}

fn effect_from_request(request: DynamicRequest) -> Result<DynamicEffect> {
    Ok(match request.mode {
        DynamicMode::Neon => DynamicEffect::Neon,
        DynamicMode::Breathing => DynamicEffect::Breathing {
            color: request.color.context("dynamic effect color is missing")?,
        },
        DynamicMode::Shifting => DynamicEffect::Shifting {
            color: request.color.context("dynamic effect color is missing")?,
            direction: request.direction.context("dynamic direction is missing")?,
        },
        DynamicMode::Wave => DynamicEffect::Wave {
            direction: request.direction.context("dynamic direction is missing")?,
        },
        DynamicMode::Zoom => DynamicEffect::Zoom {
            color: request.color.context("dynamic effect color is missing")?,
        },
    })
}

fn encode_brightness(level: Brightness) -> u64 {
    (((level.get() - 1) as u64) * 25) << 16
}

fn encode_dynamic_payload(
    request: &DynamicRequest,
    brightness: Brightness,
    store: &LightingStore,
) -> Result<u64> {
    let (selector, uses_color, uses_direction, wave_flag) = match request.mode {
        DynamicMode::Breathing => (1, true, false, 0),
        DynamicMode::Neon => (2, false, false, 0),
        DynamicMode::Wave => (3, false, true, 0x0800_0000),
        DynamicMode::Shifting => (4, true, true, 0),
        DynamicMode::Zoom => (5, true, false, 0),
    };
    let mut payload = selector as u64 | ((request.speed.get() as u64) << 8);
    payload |= (((brightness.get() - 1) as u64) * 25) << 16;
    payload |= wave_flag;
    if uses_direction {
        payload |= direction_code(request.direction.context("dynamic direction is missing")?) << 32;
    }
    if uses_color {
        let color = adjust_color(store, request.color.context("dynamic color is missing")?)?;
        payload |= (color.red as u64) << 40;
        payload |= (color.green as u64) << 48;
        payload |= (color.blue as u64) << 56;
    }
    Ok(payload)
}

fn encode_zone_behavior(zones: &[Zone; 4]) -> u64 {
    let mut value = 8u64;
    for (index, zone) in zones.iter().enumerate() {
        if zone.enabled {
            value |= 1u64 << (40 + index as u32);
        }
    }
    value
}

fn encode_zone_color(index: u8, color: Rgb, store: &LightingStore) -> Result<u64> {
    let color = adjust_color(store, color)?;
    Ok(zone_id(index)
        | ((color.red as u64) << 8)
        | ((color.green as u64) << 16)
        | ((color.blue as u64) << 24))
}

fn adjust_color(store: &LightingStore, color: Rgb) -> Result<Rgb> {
    let (red, green, blue) = store.color_adjustment()?;
    Ok(Rgb {
        red: ((color.red as f32) * red).floor().clamp(0.0, 255.0) as u8,
        green: ((color.green as f32) * green).floor().clamp(0.0, 255.0) as u8,
        blue: ((color.blue as f32) * blue).floor().clamp(0.0, 255.0) as u8,
    })
}

fn parse_lighting(root: &Element) -> Result<LightingSnapshot> {
    let key = child(root, "Key")?;
    let lighting = child(root, "LightingEffects")?;
    let pattern = child(root, "Pattern")?;
    let brightness = Brightness::new(attr_u8(lighting, "brightness")?)?;
    let mut parsed_zones = Vec::with_capacity(4);
    for index in 0..4 {
        let node = child(lighting, &format!("LightingEffects_Zone{}", index + 1))?;
        parsed_zones.push(Zone {
            enabled: attr_u8(node, "status")? != 0,
            color: Rgb::parse(attr(node, "color")?)?,
        });
    }
    let zones: [Zone; 4] = parsed_zones
        .try_into()
        .map_err(|_| anyhow::anyhow!("keyboard XML did not contain four zones"))?;
    let selected = attr(pattern, "selected")?.parse::<usize>()?;
    let selected_pattern = child(pattern, &format!("Pattern{selected}"))?;
    let dynamic = DynamicLighting {
        effect: parse_dynamic_effect(
            selected,
            pattern
                .attributes
                .get("color")
                .map(String::as_str)
                .map(Rgb::parse)
                .transpose()?,
            attr_u8(selected_pattern, "direction")?,
        )?,
        speed: DynamicSpeed::new(attr_u8(selected_pattern, "speed")?)?,
    };
    let mode = if attr_u8(key, "status")? == 0 {
        LightingMode::Static
    } else {
        LightingMode::Dynamic
    };
    Ok(LightingSnapshot {
        brightness,
        lighting: KeyboardLightingState {
            mode,
            static_zones: zones,
            dynamic,
        },
    })
}

fn parse_dynamic_effect(index: usize, color: Option<Rgb>, direction: u8) -> Result<DynamicEffect> {
    Ok(match index {
        0 => DynamicEffect::Breathing {
            color: color.context("breathing effect color is missing")?,
        },
        1 => DynamicEffect::Wave {
            direction: parse_direction(direction)?,
        },
        2 => DynamicEffect::Zoom {
            color: color.context("zoom effect color is missing")?,
        },
        3 => DynamicEffect::Shifting {
            color: color.context("shifting effect color is missing")?,
            direction: parse_direction(direction)?,
        },
        4 => DynamicEffect::Neon,
        _ => bail!("unsupported persisted dynamic pattern {index}"),
    })
}

fn update_static_xml(root: &mut Element, zones: &[Zone; 4], brightness: Brightness) -> Result<()> {
    let key = child_mut(root, "Key")?;
    set_attr(key, "status", 0);
    set_attr(key, "brightness", brightness.get());
    if let Some(color) = zones
        .iter()
        .find(|zone| zone.enabled)
        .map(|zone| zone.color)
    {
        for index in 0..COLOR_TAG_COUNT {
            if let Ok(tag) = child_mut(key, &format!("Tag{index}")) {
                tag.attributes
                    .insert("color".to_string(), color.to_string());
            }
        }
    }
    let lighting = child_mut(root, "LightingEffects")?;
    set_attr(lighting, "brightness", brightness.get());
    for (index, zone) in zones.iter().enumerate() {
        let node = child_mut(lighting, &format!("LightingEffects_Zone{}", index + 1))?;
        set_attr(node, "status", zone.enabled as u8);
        if zone.enabled {
            node.attributes
                .insert("color".to_string(), zone.color.to_string());
        }
    }
    Ok(())
}

fn update_dynamic_xml(
    root: &mut Element,
    request: DynamicRequest,
    brightness: Brightness,
) -> Result<()> {
    let pattern_index = pattern_index(request.mode);
    let key = child_mut(root, "Key")?;
    set_attr(key, "status", 1);
    set_attr(key, "brightness", brightness.get());
    if let Some(color) = request.color {
        for index in 0..COLOR_TAG_COUNT {
            if let Ok(tag) = child_mut(key, &format!("Tag{index}")) {
                tag.attributes
                    .insert("color".to_string(), color.to_string());
            }
        }
    }
    let pattern = child_mut(root, "Pattern")?;
    set_attr(pattern, "selected", pattern_index);
    if let Some(color) = request.color {
        pattern
            .attributes
            .insert("color".to_string(), color.to_string());
    }
    let selected = child_mut(pattern, &format!("Pattern{pattern_index}"))?;
    set_attr(selected, "speed", request.speed.get());
    set_attr(
        selected,
        "direction",
        request.direction.map(direction_code).unwrap_or(0) as u8,
    );
    set_attr(
        child_mut(root, "LightingEffects")?,
        "brightness",
        brightness.get(),
    );
    Ok(())
}

fn mutate_profile(
    platform: &Platform,
    mutator: impl FnOnce(&mut Element) -> Result<()>,
) -> Result<()> {
    let store = LightingStore::resolve(platform)?;
    let mut root = store.read()?;
    mutator(&mut root)?;
    parse_lighting(&root)?;
    store.write(&root, platform)
}

fn child<'a>(element: &'a Element, name: &str) -> Result<&'a Element> {
    element
        .children
        .iter()
        .find_map(|node| match node {
            XMLNode::Element(child) if child.name == name => Some(child),
            _ => None,
        })
        .with_context(|| format!("missing XML child {name}"))
}

fn child_mut<'a>(element: &'a mut Element, name: &str) -> Result<&'a mut Element> {
    element
        .children
        .iter_mut()
        .find_map(|node| match node {
            XMLNode::Element(child) if child.name == name => Some(child),
            _ => None,
        })
        .with_context(|| format!("missing XML child {name}"))
}

fn attr<'a>(element: &'a Element, name: &str) -> Result<&'a str> {
    element
        .attributes
        .get(name)
        .map(String::as_str)
        .with_context(|| format!("missing XML attribute {name}"))
}

fn attr_u8(element: &Element, name: &str) -> Result<u8> {
    Ok(attr(element, name)?.parse()?)
}

fn set_attr(element: &mut Element, name: &str, value: impl ToString) {
    element
        .attributes
        .insert(name.to_string(), value.to_string());
}

fn pattern_index(mode: DynamicMode) -> usize {
    match mode {
        DynamicMode::Breathing => 0,
        DynamicMode::Wave => 1,
        DynamicMode::Zoom => 2,
        DynamicMode::Shifting => 3,
        DynamicMode::Neon => 4,
    }
}

fn parse_direction(code: u8) -> Result<Direction> {
    match code {
        1 => Ok(Direction::FromLeft),
        2 => Ok(Direction::FromRight),
        _ => bail!("unsupported persisted dynamic direction code {code}"),
    }
}

fn direction_code(direction: Direction) -> u64 {
    match direction {
        Direction::FromLeft => 1,
        Direction::FromRight => 2,
    }
}

fn zone_id(index: u8) -> u64 {
    match index {
        1 => 1,
        2 => 2,
        3 => 4,
        4 => 8,
        _ => 0,
    }
}

fn ensure_success(command: u16, return_code: u32) -> Result<()> {
    if return_code != 0 {
        bail!("PredatorSense command {command} failed with return code {return_code}")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use xmltree::Element;

    #[test]
    fn encodes_brightness() {
        assert_eq!(encode_brightness(Brightness::new(1).unwrap()), 0);
        assert_eq!(encode_brightness(Brightness::new(5).unwrap()), 100 << 16);
    }

    #[test]
    fn parses_typed_static_lighting_with_four_zones() {
        let xml = r##"
<ROOT><Key status="0" brightness="3"/><Pattern selected="1"><Pattern1 speed="5" direction="2"/></Pattern><LightingEffects brightness="3">
<LightingEffects_Zone1 status="1" color="#FF0000"/><LightingEffects_Zone2 status="0" color="#00FF00"/>
<LightingEffects_Zone3 status="1" color="#0000FF"/><LightingEffects_Zone4 status="0" color="#FFFFFF"/>
</LightingEffects></ROOT>"##;
        let parsed = parse_lighting(&Element::parse(xml.as_bytes()).unwrap()).unwrap();
        assert_eq!(parsed.brightness.get(), 3);
        assert_eq!(parsed.lighting.mode, LightingMode::Static);
        assert!(parsed.lighting.static_zones[0].enabled);
        assert!(!parsed.lighting.static_zones[1].enabled);
        assert_eq!(parsed.lighting.dynamic.speed.get(), 5);
        assert_eq!(
            parsed.lighting.dynamic.effect,
            DynamicEffect::Wave {
                direction: Direction::FromRight,
            }
        );
    }

    #[test]
    fn wave_does_not_accept_or_require_a_color() {
        let request = DynamicRequest::new(
            DynamicMode::Wave,
            DynamicSpeed::new(1).unwrap(),
            None,
            Some(Direction::FromLeft),
        )
        .unwrap();
        assert_eq!(
            effect_from_request(request).unwrap(),
            DynamicEffect::Wave {
                direction: Direction::FromLeft,
            }
        );
        assert!(
            DynamicRequest::new(
                DynamicMode::Wave,
                DynamicSpeed::new(1).unwrap(),
                Some(Rgb::parse("00FFFF").unwrap()),
                Some(Direction::FromLeft),
            )
            .is_err()
        );
    }

    #[test]
    fn neon_is_a_colorless_dynamic_effect() {
        let request =
            DynamicRequest::new(DynamicMode::Neon, DynamicSpeed::new(1).unwrap(), None, None)
                .unwrap();
        assert_eq!(effect_from_request(request).unwrap(), DynamicEffect::Neon);
    }
}
