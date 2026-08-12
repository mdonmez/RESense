# Device Protocol Reference

Maintainer reference for the vendor command surface used by the current device layer. These details are implementation data and are not part of the public CLI or status output.

## Transport

- Service pipe: `\\.\pipe\PredatorSense_service_namedpipe`
- Admin-agent pipe prefix: `\\.\pipe\PredatorSense_admin_agent_<session_id>`

Each request contains:

1. A little-endian `u16` command code.
2. A `u8` argument count.
3. For each argument, a `u32` byte length followed by the argument bytes.

The current adapter expects nine-byte setter replies, nine-byte `u32` getter replies, and thirteen-byte `u64` getter replies. It validates each frame and decodes the result before returning it to the device layer.

## Commands

| Code | Channel | Device operation |
| ---: | --- | --- |
| `2` | Admin | Sticky Keys |
| `3` | Service | Persist a vendor DWORD setting |
| `9` | Service | Windows/Menu lock and LCD overdrive |
| `10` | Service | Windows/Menu lock and LCD overdrive readback |
| `13` | Service | Fan temperature and RPM readback |
| `13` | Admin | DTS sound preset readback |
| `14` | Admin | DTS sound preset write |
| `15` | Service | Fan control mode |
| `15` | Admin | WhisperMode |
| `16` | Service | Manual fan percentage |
| `17` | Service | Keyboard backlight timeout write |
| `20` | Service | Keyboard backlight timeout readback |
| `27` | Service | Keyboard brightness and dynamic lighting |
| `28` | Service | Static keyboard zone colors |
| `29` | Service | Static keyboard zone enablement |
| `30` | Service | Operation mode write |
| `34` | Service | Operation mode readback |

## Payloads

### Windows/Menu Lock And LCD Overdrive

Command `9` receives one `u64` payload:

- Windows/Menu lock: selector `2`, state at bit shift `24`.
- LCD overdrive: selector `0x10`, state at bit shift `48`.

Command `10` query `0` returns the corresponding state fields:

- Windows/Menu lock: `((value >> 24) & 0xFF) == 1`.
- LCD overdrive: `((value >> 48) & 0xFF) == 1`.

### Fan Telemetry

Command `13` uses a query value of `1 | (index << 8)`:

- Index `1`: CPU temperature.
- Index `2`: CPU fan RPM.
- Index `6`: GPU fan RPM.
- Index `10`: GPU temperature.

The low byte is the vendor status and the next sixteen bits contain the reading.

### DTS Sound

Admin command `14` receives one preset code:

| Preset | Code |
| --- | ---: |
| music | `0` |
| movies | `1` |
| voice | `2` |
| strategy | `3` |
| rpg | `4` |
| shooter | `5` |
| custom | `6` |
| auto | `10` |

The result is verified by reading the preset with admin command `13`.

### Fan Control

Command `15` receives one `u64` behavior payload:

- Global automatic control: `0x410009`.
- Maximum control: `0x820009`.
- Custom control uses base value `9`, with CPU automatic/manual values `1`/`3` and GPU automatic/manual values `0x40`/`0xC0` at shift `16`.

Command `16` receives one manual speed payload:

- CPU selector: `1`.
- GPU selector: `4`.
- Percentage at shift `8`.

### Keyboard Backlight Timeout

Command `20` reads the timeout through:

```text
1 | (BK_Hotkey_Number << 8) | 0x80000
```

The returned value contains the current brightness byte at shift `32` and timeout byte at shift `40`. Command `17` writes the same fields using a base value of `2 | (BK_Hotkey_Number << 8) | 0x80000`. RESense preserves the current brightness byte and changes the timeout byte between `0` and `30` seconds.

### Keyboard Lighting

Command `27` handles keyboard brightness and dynamic effects. Its payload contains the effect selector, speed, brightness, optional direction, and optional RGB color.

Dynamic effect selectors are:

| Effect | Selector |
| --- | ---: |
| breathing | `1` |
| neon | `2` |
| wave | `3` |
| shifting | `4` |
| zoom | `5` |

The dynamic payload stores speed at shift `8`, brightness at shift `16`, direction at shift `32` when used, and RGB components at shifts `40`, `48`, and `56` when used.

Command `28` writes a static zone color. Zone selectors are `1`, `2`, `4`, and `8` for zones one through four. RGB components are stored at shifts `8`, `16`, and `24`. Color adjustment values come from the installed NitroSense `HW_Support.ini` file.

Command `29` writes static zone enablement. Zone enable bits are `1 << 40` through `1 << 43`.

The installed NitroSense `Main.xml` profile is updated alongside these service operations and is the read source for keyboard lighting state.

### Operation Mode

Command `30` receives the operation mode code:

| Mode | Code |
| --- | ---: |
| quiet | `0` |
| default | `1` |
| performance | `4` |

Command `34` query `11` returns the live mode code. A mode write is considered successful only after this readback matches the requested mode.

## Admin Scope

- Sticky Keys uses the admin agent for the current Windows session.
- DTS sound and WhisperMode use a reachable admin agent because their state is shared by the validated system.

The device layer owns session discovery and admin selection. Session IDs are not exposed as product state.
