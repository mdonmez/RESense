# RESense Pipe Command Catalog

This file catalogs the command surface currently used by RESense.

Transport:

- service pipe: `\\.\pipe\PredatorSense_service_namedpipe`
- admin-agent pipe prefix: `\\.\pipe\PredatorSense_admin_agent_<session_id>`

Message format:

1. `u16` command code, little-endian
2. `u8` argument count
3. for each argument:
   - `u32` byte length
   - raw argument bytes

Reply sizes:

- service/admin set replies in current RESense paths: `9` bytes
- service get replies in current RESense paths: `13` bytes

Current decoding convention:

- set reply `raw[5..9]` -> `u32 return_code`
- get reply `raw[5..13]` -> `u64 value`

## Command Inventory

| Code | Transport | Purpose in RESense | Request shape | Reply shape | Status |
| --- | --- | --- | --- | --- | --- |
| `2` | admin | sticky keys | one `u32` bool | fire-and-forget | understood for supported path |
| `3` | service | HKLM write helper | path string, name string, type `u32`, value `u32` | set return code | understood for supported path |
| `9` | service | gaming profile setters: Win/Menu lock and LCD overdrive | one `u64` payload | set return code | understood for supported path |
| `10` | service | gaming profile getter | one `u32` query | `u64` value | partially understood; supported decodes only |
| `13` | service | fan thermals/RPM getter | one `u32` query | `u64` value | understood for supported queries |
| `14` | admin | DTS sound preset setter | one `u32` mode code | fire-and-forget | understood for supported path |
| `15` | service | fan behavior setter | one `u64` payload | set return code | understood for supported path |
| `15` | admin | WhisperMode setter | one `u32` bool | fire-and-forget | understood for supported path |
| `16` | service | fan speed setter | one `u64` payload | set return code | understood for supported path |
| `17` | service | generic WMI setter; used for keyboard backlight timeout | one `u64` payload | set return code | understood for supported path |
| `20` | service | generic WMI getter; used for keyboard backlight timeout | one `u32` query | `u64` value | partially understood; supported decodes only |
| `27` | service | keyboard backlight / dynamic payload setter | one `u64` payload | set return code | understood for supported path |
| `28` | service | keyboard per-zone RGB setter | one `u64` payload | set return code | understood for supported path |
| `29` | service | keyboard per-zone behavior setter | one `u64` payload | set return code | understood for supported path |
| `30` | service | operation mode setter | one `u32` mode code | set return code | understood for supported path; reply is non-authoritative |
| `34` | service | gaming misc getter; used for operation mode | one `u32` query | `u64` value | understood for supported query `11` |

## Supported Command Details

### `cmd 9` service set

Used for:

- Win/Menu key lock
- LCD overdrive

Payloads:

- Win/Menu key lock:
  - selector `2`
  - enabled byte at shift `24`
  - formula: `2 | ((enabled as u64) << 24)`
- LCD overdrive:
  - selector `0x10`
  - enable bit at `1 << 48`
  - formula: `0x10 | (enabled ? 1<<48 : 0)`

Expected reply:

- set reply with `return_code == 0`

### `cmd 10` service get

Supported decodes:

- query `0`:
  - fan diagnostic payload only; no longer authoritative for supported fan mode
  - Win/Menu key lock readback uses `((value >> 24) & 0xFF) == 1`
  - LCD overdrive readback uses `((value >> 48) & 0xFF) == 1`

Known raw reply example:

- query `0` baseline raw: `010800000000ff010100ff0100`

Partial / unsupported:

- other query ids mostly collapse to non-success or undifferentiated results
- not a trustworthy keyboard live-state getter

### `cmd 13` service get

Used for fan thermals and RPM:

- query formula: `1 | (index << 8)`

Supported indexes:

- `1` -> CPU temp, query `0x0101`
- `2` -> CPU fan RPM, query `0x0201`
- `6` -> GPU fan RPM, query `0x0601`
- `10` -> GPU temp, query `0x0A01`

Decode:

- status byte: `value & 0xFF`
- value field: `((value >> 8) & 0xFFFF) as u16`

### `cmd 14` admin set

Used for DTS sound preset writes.

Mode codes:

- `0 music`
- `1 movies`
- `2 voice`
- `3 strategy`
- `4 rpg`
- `5 shooter`
- `6 custom`
- `10 auto`

Verification:

- re-read via admin `cmd 13`

### `cmd 15` service set

Used for fan behavior.

Supported payloads:

- global auto: `0x410009`
- max: `0x820009`
- exact custom behavior:
  - base low bits: `9`
  - CPU auto/manual byte contributes `1` or `3` at shift `16`
  - GPU auto/manual byte contributes `0x40` or `0xC0` at shift `16`

Examples:

- custom all auto: `0x410009`
- custom CPU manual / GPU auto: `0x430009`

### `cmd 16` service set

Used for manual custom fan speeds.

Payload:

- CPU selector `1`
- GPU selector `4`
- percent at shift `8`
- formula: `selector | ((percent as u64) << 8)`

### `cmd 17` / `cmd 20` generic WMI

Supported path:

- keyboard backlight timeout

`cmd 20` get payload:

- formula: `1 | (bk_hotkey_number << 8) | 0x80000`

Decode:

- brightness byte: `(value >> 32) & 0xFF`
- timeout byte: `(value >> 40) & 0xFF`

`cmd 17` set payload:

- formula:
  - base: `2 | (bk_hotkey_number << 8) | 0x80000`
  - brightness byte at shift `32`
  - timeout byte at shift `40`

Supported contract:

- timeout byte `0` or `30`
- brightness byte is intentionally not part of supported display/keyboard brightness state

Known raw getter examples:

- timeout off example: `0108000000000f1e0000000000`
- timeout on/off interpretation depends on the decoded timeout byte, not the whole raw frame

Partial / unsupported:

- many other successful query ids exist in `cmd 20`, but they are not yet part of the supported contract

### `cmd 27` service set

Used for:

- static brightness changes
- dynamic effect payloads
- leaving dynamic mode by writing a brightness-only payload before static writes

Static brightness payload:

- formula: `(((level - 1) as u64) * 25) << 16`

Dynamic payload fields:

- selector in low byte:
  - `1 breathing`
  - `2 neon`
  - `3 wave`
  - `4 shifting`
  - `5 zoom`
- speed at shift `8`
- brightness percentage bucket at shift `16`
- direction code at shift `32` when applicable
  - `1 from-left`
  - `2 from-right`
- RGB at shifts `40`, `48`, `56` when applicable
- wave additionally sets `0x0800_0000`

### `cmd 28` service set

Used for static enabled-zone colors.

Payload:

- zone selector low byte:
  - zone 1 -> `1`
  - zone 2 -> `2`
  - zone 3 -> `4`
  - zone 4 -> `8`
- adjusted RGB bytes at shifts `8`, `16`, `24`

Color adjustment source:

- `references\NitroSense\NitroSense\HW_Support.ini`
- section `[ZoneColorAdjust]`

### `cmd 29` service set

Used for static per-zone enabled/disabled behavior.

Payload:

- base `8`
- zone enable bits:
  - zone 1 -> `1 << 40`
  - zone 2 -> `1 << 41`
  - zone 3 -> `1 << 42`
  - zone 4 -> `1 << 43`

### `cmd 30` service set

Used for operation mode.

Mode codes:

- `0 quiet`
- `1 default`
- `4 performance`

Known behavior on this machine:

- the numeric reply from `cmd 30` is not authoritative enough to represent success or failure by itself
- observed raw reply for all tested writes in this environment: `0104000000ffffffff`
- observed `return_code`: `u32::MAX`
- despite the identical reply:
  - `default -> quiet` changed the live mode to `quiet`
  - `quiet -> default` changed the live mode to `default`
  - `default -> default` was a no-op and stayed `default`
  - `default -> performance` initially stayed `default`
- NitroSense's managed UI also ignores the numeric result from `set_operation_mode()` and updates its UI/registry before firing the service write
- supported RESense contract:
  - send `cmd 30`
  - verify success only through `cmd 34/query 11`
  - treat the `cmd 30` reply as transport/debug data, not as authoritative state

### `cmd 34` service get

Supported decode:

- query `11`
- status byte: `value & 0xFF`
- mode byte: `(value >> 8) & 0xFF`

Validated mode codes:

- `0 quiet`
- `1 default`
- `4 performance`

## Partially Understood / Diagnostic Paths

| Command | Path | Current conclusion |
| --- | --- | --- |
| `10/query 0` | fan diagnostic payload | not authoritative for supported fan active mode |
| `12/*` | keyboard LED group getter | not a trustworthy live keyboard-state source on this machine |
| `20/*` outside backlight timeout payload | generic WMI getter | many successful queries exist, but their meaning is not yet part of the supported contract |
| `34/*` outside query `11` | gaming misc getter | other successful queries exist, but are not yet part of the supported contract |

## Session / Admin Notes

- Admin commands iterate candidate session ids from `src/platform/session.rs`.
- Current admin-transport users:
  - sticky keys
  - WhisperMode
  - DTS sound getter/setter
- Verified on 2026-05-18 for sticky keys:
  - with `PredatorSense_admin_agent_4` and `PredatorSense_admin_agent_5` both alive, direct writes to each pipe affected different Windows sessions
  - normal `resense keyboard sticky enable` from session `4` changed the current session only
  - `HKLM\SOFTWARE\OEM\NitroSense\AdvanceSettings\StickyKey` is not a reliable cross-session truth source by itself
- Verified on 2026-05-18 for DTS sound:
  - with `PredatorSense_admin_agent_4` and `PredatorSense_admin_agent_5` both alive, direct `cmd 14` write to pipe `5` changed the visible NitroSense sound preset in both sessions
  - normal `resense sound music` from session `4` also changed the visible preset in both sessions
  - current conclusion: admin transport is session-addressed, but the DTS sound state it controls is effectively shared/global on this machine
- WhisperMode multi-session behavior is not yet fully documented; current RESense behavior is still "first working session wins".
