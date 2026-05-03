use crate::cli::{Direction, KeyboardDynamicArgs, KeyboardDynamicMode, KeyboardStaticArgs};
use crate::error::Result;
use crate::platform::{pipe, registry, session, sticky_keys};
use anyhow::{Context, bail};
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use xmltree::{Element, XMLNode};

const CMD_SET_KB_BACKLIGHT: u16 = 27;
const CMD_SET_RGB_KB: u16 = 28;
const CMD_SET_LED_BEHAVIOR: u16 = 29;
const CMD_GET_LED_GROUP_COLOR: u16 = 12;
const CMD_ADMIN_SET_STICKY_KEYS: u16 = 2;
const CMD_SET_GAMING_PROFILE: u16 = 9;
const CMD_GET_GAMING_PROFILE: u16 = 10;
const QUERY_GAMING_PROFILE: u32 = 0;
const WIN_MENU_SELECTOR: u64 = 2;
const WIN_MENU_STATUS_SHIFT: u64 = 24;
const COLOR_TAG_COUNT: usize = 127;
const SYSTEM_PROFILE_ROOT: &str = r"C:\ProgramData\OEM\NitroSense\ProfilePool\LightProfilePool";
const HW_SUPPORT_INI: &str = r"references\NitroSense\NitroSense\HW_Support.ini";

#[derive(Debug, Clone, Serialize)]
pub struct ZoneState {
    pub index: u8,
    pub status: bool,
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DynamicState {
    pub mode: String,
    pub speed: u8,
    pub brightness: u8,
    pub color: Option<String>,
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyboardState {
    pub mode: String,
    pub brightness: u8,
    pub static_zones: Vec<ZoneState>,
    pub dynamic: Option<DynamicState>,
    pub profile_xml: String,
}

pub fn set_brightness(level: u8) -> Result<()> {
    let encoded = encode_brightness(level);
    let (raw, return_code) = pipe::service_set(CMD_SET_KB_BACKLIGHT, &[pipe::u64_arg(encoded)])?;
    ensure_success(CMD_SET_KB_BACKLIGHT, &raw, return_code)?;
    sync_brightness_to_system_xml(level)?;
    Ok(())
}

pub fn set_static(args: &KeyboardStaticArgs) -> Result<KeyboardState> {
    let zones = build_zones(args)?;
    set_brightness(args.brightness)?;
    let behavior = encode_zone_behavior(&zones);
    let (raw, return_code) = pipe::service_set(CMD_SET_LED_BEHAVIOR, &[pipe::u64_arg(behavior)])?;
    ensure_success(CMD_SET_LED_BEHAVIOR, &raw, return_code)?;

    for zone in &zones {
        if zone.status {
            let encoded = encode_zone_color(zone.index, &zone.color)?;
            let (raw, return_code) = pipe::service_set(CMD_SET_RGB_KB, &[pipe::u64_arg(encoded)])?;
            ensure_success(CMD_SET_RGB_KB, &raw, return_code)?;
        }
    }

    sync_static_to_system_xml(&zones, args.brightness)
}

pub fn set_dynamic(args: &KeyboardDynamicArgs) -> Result<KeyboardState> {
    let payload = encode_dynamic_payload(args)?;
    let (raw, return_code) = pipe::service_set(CMD_SET_KB_BACKLIGHT, &[pipe::u64_arg(payload)])?;
    ensure_success(CMD_SET_KB_BACKLIGHT, &raw, return_code)?;
    sync_dynamic_to_system_xml(args)
}

pub fn set_sticky_keys(enabled: bool) -> Result<()> {
    let mut last_error = None;
    for session_id in session::candidate_session_ids() {
        let pipe_name = session::admin_pipe_name(session_id);
        match pipe::send_fire_and_forget(&pipe_name, CMD_ADMIN_SET_STICKY_KEYS, &[pipe::u32_arg(enabled as u32)]) {
            Ok(()) => {
                registry::set_hklm_dword(registry::ADVANCED_SETTINGS, "StickyKey", enabled as u32)?;
                return Ok(());
            }
            Err(error) => last_error = Some(error),
        }
    }
    match last_error {
        Some(error) => Err(error),
        None => bail!("no admin-agent session candidates"),
    }
}

pub fn set_win_menu_lock(enabled: bool) -> Result<()> {
    let payload = WIN_MENU_SELECTOR | ((enabled as u64) << WIN_MENU_STATUS_SHIFT);
    let (raw, return_code) = pipe::service_set(CMD_SET_GAMING_PROFILE, &[pipe::u64_arg(payload)])?;
    ensure_success(CMD_SET_GAMING_PROFILE, &raw, return_code)?;
    registry::set_hklm_dword(registry::ADVANCED_SETTINGS, "WinAndMenuKey", enabled as u32)?;
    Ok(())
}

pub fn read_state() -> Result<KeyboardState> {
    parse_profile_state(&system_profile_xml_path()?)
}

pub fn read_zone_statuses() -> Result<Vec<ZoneState>> {
    let mut result = Vec::new();
    for (index, zone_id) in [(1, 1u32), (2, 2), (3, 4), (4, 8)] {
        let (_, value) = pipe::service_get_u64(CMD_GET_LED_GROUP_COLOR, &[pipe::u32_arg(zone_id)])?;
        result.push(ZoneState {
            index,
            status: (value & 0xFF) == 0,
            color: "live-status-only".to_string(),
        });
    }
    Ok(result)
}

pub fn read_sticky_keys() -> Result<sticky_keys::StickyKeysState> {
    sticky_keys::read()
}

pub fn read_win_menu_lock() -> Result<bool> {
    let (_, value) = pipe::service_get_u64(CMD_GET_GAMING_PROFILE, &[pipe::u32_arg(QUERY_GAMING_PROFILE)])?;
    Ok(((value >> WIN_MENU_STATUS_SHIFT) & 0xFF) == 1)
}

pub fn encode_brightness(level: u8) -> u64 {
    (((level - 1) as u64) * 25) << 16
}

pub fn encode_dynamic_payload(args: &KeyboardDynamicArgs) -> Result<u64> {
    let (selector, uses_color, uses_direction, wave_flag) = match args.mode {
        KeyboardDynamicMode::Breathing => (1, true, false, 0),
        KeyboardDynamicMode::Neon => (2, false, false, 0),
        KeyboardDynamicMode::Wave => (3, true, true, 0x0800_0000),
        KeyboardDynamicMode::Shifting => (4, true, true, 0),
        KeyboardDynamicMode::Zoom => (5, true, false, 0),
    };

    let mut payload = selector as u64;
    payload |= (args.speed as u64) << 8;
    payload |= (((args.brightness - 1) as u64) * 25) << 16;
    payload |= wave_flag;

    if uses_direction {
        let direction = args.direction.context("dynamic mode requires direction")?;
        payload |= direction_code(direction) << 32;
    }

    if uses_color {
        let color = args.color.as_deref().context("dynamic mode requires color")?;
        let (red, green, blue) = adjusted_rgb(color)?;
        payload |= (red as u64) << 40;
        payload |= (green as u64) << 48;
        payload |= (blue as u64) << 56;
    }

    Ok(payload)
}

fn build_zones(args: &KeyboardStaticArgs) -> Result<Vec<ZoneState>> {
    let values = [
        (1, args.zone1.as_deref()),
        (2, args.zone2.as_deref()),
        (3, args.zone3.as_deref()),
        (4, args.zone4.as_deref()),
    ];
    values
        .into_iter()
        .filter_map(|(index, color)| color.map(|color| (index, color)))
        .map(|(index, color)| {
            let normalized = normalize_color(color)?;
            Ok(ZoneState {
                index,
                status: normalized != "off",
                color: normalized,
            })
        })
        .collect()
}

fn encode_zone_behavior(zones: &[ZoneState]) -> u64 {
    let mut value = 8u64;
    for zone in zones {
        if zone.status {
            value |= 1u64 << (39 + zone.index);
        }
    }
    value
}

fn encode_zone_color(zone_index: u8, color: &str) -> Result<u64> {
    let (red, green, blue) = adjusted_rgb(color)?;
    Ok(zone_id(zone_index) | ((red as u64) << 8) | ((green as u64) << 16) | ((blue as u64) << 24))
}

fn adjusted_rgb(color: &str) -> Result<(u8, u8, u8)> {
    let (red, green, blue) = parse_rgb(color)?;
    let (r_adj, g_adj, b_adj) = read_zone_color_adjustment().unwrap_or((1.0, 1.0, 1.0));
    Ok((
        ((red as f32) * r_adj).floor().clamp(0.0, 255.0) as u8,
        ((green as f32) * g_adj).floor().clamp(0.0, 255.0) as u8,
        ((blue as f32) * b_adj).floor().clamp(0.0, 255.0) as u8,
    ))
}

fn normalize_color(value: &str) -> Result<String> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("off") {
        return Ok("off".to_string());
    }
    let value = value.strip_prefix('#').unwrap_or(value);
    if value.len() != 6 || !value.chars().all(|char| char.is_ascii_hexdigit()) {
        bail!("expected RRGGBB color or off, got {value}");
    }
    Ok(format!("#{}", value.to_ascii_uppercase()))
}

fn parse_rgb(value: &str) -> Result<(u8, u8, u8)> {
    let color = normalize_color(value)?;
    if color == "off" {
        bail!("off is not a valid RGB payload");
    }
    Ok((
        u8::from_str_radix(&color[1..3], 16)?,
        u8::from_str_radix(&color[3..5], 16)?,
        u8::from_str_radix(&color[5..7], 16)?,
    ))
}

fn read_zone_color_adjustment() -> Result<(f32, f32, f32)> {
    let text = std::fs::read_to_string(HW_SUPPORT_INI).context("reading HW_Support.ini")?;
    let mut in_section = false;
    let mut values = (1.0, 1.0, 1.0);
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == "[ZoneColorAdjust]";
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let parsed = value.trim().parse::<f32>().unwrap_or(1.0);
            match key.trim() {
                "R" => values.0 = parsed,
                "G" => values.1 = parsed,
                "B" => values.2 = parsed,
                _ => {}
            }
        }
    }
    Ok(values)
}

fn active_profile_name() -> Result<String> {
    Ok(registry::read_hklm_string(registry::LIGHT_SETTING, "LightingProfile")
        .unwrap_or_else(|_| "Default".to_string()))
}

fn system_profile_xml_path() -> Result<PathBuf> {
    Ok(PathBuf::from(SYSTEM_PROFILE_ROOT).join(active_profile_name()?).join("Main.xml"))
}

fn parse_profile_state(path: &Path) -> Result<KeyboardState> {
    let root = read_xml(path)?;
    let key = child(&root, "Key")?;
    let lighting = child(&root, "LightingEffects")?;
    let pattern = child(&root, "Pattern")?;
    let mode = if attr_u8(key, "status")? == 0 { "static" } else { "dynamic" };
    let brightness = attr_u8(lighting, "brightness")?;
    let mut static_zones = Vec::new();
    for index in 1..=4 {
        let node = child(lighting, &format!("LightingEffects_Zone{index}"))?;
        static_zones.push(ZoneState {
            index,
            status: attr_u8(node, "status")? != 0,
            color: normalize_color(attr(node, "color")?)?,
        });
    }
    let selected = attr(pattern, "selected")?.parse::<usize>().unwrap_or(0);
    let dynamic = if mode == "dynamic" {
        let selected_pattern = child(pattern, &format!("Pattern{selected}"))?;
        Some(DynamicState {
            mode: pattern_name(selected).to_string(),
            speed: attr_u8(selected_pattern, "speed")?,
            brightness: attr_u8(key, "brightness")?,
            color: Some(normalize_color(attr(pattern, "color")?)?),
            direction: Some(direction_name(attr_u8(selected_pattern, "direction")?).to_string()),
        })
    } else {
        None
    };

    Ok(KeyboardState {
        mode: mode.to_string(),
        brightness,
        static_zones,
        dynamic,
        profile_xml: path.display().to_string(),
    })
}

fn sync_brightness_to_system_xml(brightness: u8) -> Result<KeyboardState> {
    mutate_profile_xml(|root| {
        child_mut(root, "Key")?.attributes.insert("brightness".to_string(), brightness.to_string());
        child_mut(root, "LightingEffects")?.attributes.insert("brightness".to_string(), brightness.to_string());
        Ok(())
    })
}

fn sync_static_to_system_xml(zones: &[ZoneState], brightness: u8) -> Result<KeyboardState> {
    mutate_profile_xml(|root| {
        let key = child_mut(root, "Key")?;
        key.attributes.insert("status".to_string(), "0".to_string());
        key.attributes.insert("brightness".to_string(), brightness.to_string());
        if let Some(first_enabled) = zones.iter().find(|zone| zone.status).map(|zone| zone.color.clone()) {
            for index in 0..COLOR_TAG_COUNT {
                if let Ok(tag) = child_mut(key, &format!("Tag{index}")) {
                    tag.attributes.insert("color".to_string(), first_enabled.clone());
                }
            }
        }
        let lighting = child_mut(root, "LightingEffects")?;
        lighting.attributes.insert("brightness".to_string(), brightness.to_string());
        for zone in zones {
            let node = child_mut(lighting, &format!("LightingEffects_Zone{}", zone.index))?;
            node.attributes.insert("status".to_string(), (zone.status as u8).to_string());
            if zone.status {
                node.attributes.insert("color".to_string(), zone.color.clone());
            }
        }
        Ok(())
    })
}

fn sync_dynamic_to_system_xml(args: &KeyboardDynamicArgs) -> Result<KeyboardState> {
    mutate_profile_xml(|root| {
        let pattern_index = pattern_index(args.mode);
        let key = child_mut(root, "Key")?;
        key.attributes.insert("status".to_string(), "1".to_string());
        key.attributes.insert("brightness".to_string(), args.brightness.to_string());
        if let Some(color) = &args.color {
            let normalized = normalize_color(color)?;
            for index in 0..COLOR_TAG_COUNT {
                if let Ok(tag) = child_mut(key, &format!("Tag{index}")) {
                    tag.attributes.insert("color".to_string(), normalized.clone());
                }
            }
        }
        let pattern = child_mut(root, "Pattern")?;
        pattern.attributes.insert("selected".to_string(), pattern_index.to_string());
        if let Some(color) = &args.color {
            pattern.attributes.insert("color".to_string(), normalize_color(color)?);
        }
        let selected = child_mut(pattern, &format!("Pattern{pattern_index}"))?;
        selected.attributes.insert("speed".to_string(), args.speed.to_string());
        selected.attributes.insert(
            "direction".to_string(),
            args.direction.map(xml_direction_code).unwrap_or(0).to_string(),
        );
        child_mut(root, "LightingEffects")?
            .attributes
            .insert("brightness".to_string(), args.brightness.to_string());
        Ok(())
    })
}

fn mutate_profile_xml(mutator: impl FnOnce(&mut Element) -> Result<()>) -> Result<KeyboardState> {
    let path = system_profile_xml_path()?;
    let mut root = read_xml(&path)?;
    mutator(&mut root)?;
    let mut file = File::create(&path).with_context(|| format!("opening {}", path.display()))?;
    root.write(&mut file).with_context(|| format!("writing {}", path.display()))?;
    parse_profile_state(&path)
}

fn read_xml(path: &Path) -> Result<Element> {
    Element::parse(File::open(path).with_context(|| format!("opening {}", path.display()))?)
        .with_context(|| format!("parsing {}", path.display()))
}

fn child<'a>(element: &'a Element, name: &str) -> Result<&'a Element> {
    element.children.iter().find_map(|node| match node {
        XMLNode::Element(child) if child.name == name => Some(child),
        _ => None,
    }).with_context(|| format!("missing XML child {name}"))
}

fn child_mut<'a>(element: &'a mut Element, name: &str) -> Result<&'a mut Element> {
    element.children.iter_mut().find_map(|node| match node {
        XMLNode::Element(child) if child.name == name => Some(child),
        _ => None,
    }).with_context(|| format!("missing XML child {name}"))
}

fn attr<'a>(element: &'a Element, name: &str) -> Result<&'a str> {
    element.attributes.get(name).map(String::as_str).with_context(|| format!("missing XML attribute {name}"))
}

fn attr_u8(element: &Element, name: &str) -> Result<u8> {
    Ok(attr(element, name)?.parse()?)
}

fn pattern_index(mode: KeyboardDynamicMode) -> usize {
    match mode {
        KeyboardDynamicMode::Breathing => 0,
        KeyboardDynamicMode::Wave => 1,
        KeyboardDynamicMode::Zoom => 2,
        KeyboardDynamicMode::Shifting => 3,
        KeyboardDynamicMode::Neon => 4,
    }
}

fn pattern_name(index: usize) -> &'static str {
    match index {
        0 => "breathing",
        1 => "wave",
        2 => "zoom",
        3 => "shifting",
        4 => "neon",
        _ => "unknown",
    }
}

fn direction_code(direction: Direction) -> u64 {
    match direction {
        Direction::Left => 1,
        Direction::Right => 2,
    }
}

fn xml_direction_code(direction: Direction) -> u8 {
    direction_code(direction) as u8
}

fn direction_name(code: u8) -> &'static str {
    match code {
        1 => "left",
        2 => "right",
        0 => "none",
        _ => "unknown",
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

fn ensure_success(cmd: u16, raw: &[u8], return_code: u32) -> Result<()> {
    if return_code != 0 {
        bail!("PredatorSense command {cmd} failed with return_code={return_code} reply={}", hex(raw));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_brightness() {
        assert_eq!(encode_brightness(1), 0);
        assert_eq!(encode_brightness(5), 100 << 16);
    }

    #[test]
    fn parses_color() {
        assert_eq!(normalize_color("ff00aa").unwrap(), "#FF00AA");
        assert_eq!(normalize_color("off").unwrap(), "off");
    }
}
