# Linux Port Notes

## Summary

RESense can be ported to Linux for the core target scope:

- fan control
- operation mode control
- fan RPM and temperature readback
- keyboard lighting writes
- keyboard timeout if the EC mapping is valid

This is not a direct port of the current backend. The current RESense implementation is a Windows NitroSense client. Linux requires a different transport and state model.

## Current Windows Model

The current Windows implementation depends on:

- PredatorSense/NitroSense named pipes
- NitroSense-owned HKLM registry state
- NitroSense keyboard XML under `C:\ProgramData\OEM\NitroSense`
- some Win32 APIs for Windows-only features

That architecture does not exist on Linux.

## Linux Model

On Linux, RESense would need to talk to Linux-native hardware interfaces:

- EC access for fan, mode, RPM, temperatures, and possibly keyboard timeout
- kernel device nodes or kernel modules for keyboard RGB control
- possibly `acer-wmi`, sysfs, debugfs, or another WMI-backed interface for features like LCD overdrive if such support exists

Conceptually:

- Windows: `RESense -> named pipes -> Acer service -> hardware`
- Linux: `RESense -> EC/device/WMI kernel interface -> hardware`

## What Linux-NitroSense Shows

The `Linux-NitroSense` repository is useful because it shows that AN515-58 hardware is reachable on Linux.

It is not architecturally similar to RESense:

- it does not use a vendor service layer
- it does not model NitroSense registry or XML state
- it reads and writes EC registers directly
- it uses a separate kernel module/device path for keyboard RGB

### Fan and Mode

`Linux-NitroSense` maps `Nitro AN515-58` to the same EC register set used for several nearby Nitro models.

Important register assumptions there:

- GPU fan mode control: `0x21`
- CPU fan mode control: `0x22`
- GPU manual fan speed: `0x3A`
- CPU manual fan speed: `0x37`
- CPU fan RPM bytes: `0x13`, `0x14`
- GPU fan RPM bytes: `0x15`, `0x16`
- CPU temp: `0xB0`
- GPU temp: `0xB6`
- system temp: `0xB3`
- keyboard timeout: `0x06`
- operation mode: `0x2C`

Mode values used there:

- quiet: `0x00`
- default: `0x01`
- extreme/performance: `0x04`

Those mode codes align with the current RESense Windows model.

### Keyboard

`Linux-NitroSense` uses device nodes exposed by a keyboard module:

- `/dev/acer-gkbbl-0`
- `/dev/acer-gkbbl-static-0`

That project demonstrates keyboard writes, not a strong live readback model.

## Linux Source Of Truth Strategy

If Linux does not provide a NitroSense-style persisted state model, RESense should use a mixed strategy:

- use live getters where Linux exposes reliable readback
- use RESense-managed internal persisted state where Linux exposes only write paths

This is acceptable as long as each feature clearly declares whether its read model is:

- live
- persisted
- validated combination

## Expected Linux Support By Feature

### Likely Properly Supported

- CPU fan RPM live readback
- GPU fan RPM live readback
- CPU temperature live readback
- GPU temperature live readback
- current high-level operation mode readback
- fan mode writes
- operation mode writes
- manual fan speed writes

### Likely Supported But Needs Validation

- current per-fan mode exact readback
- mixed custom fan states
- keyboard timeout read/write via EC
- keyboard RGB writes via kernel device/module

### Not Yet Proven

- keyboard live readback for current effect, color, brightness, and direction
- LCD overdrive on Linux

### Intentionally Out Of Scope On Linux

- Sticky Keys
- Win/Menu key lock
- NitroSense registry synchronization
- NitroSense XML synchronization
- WhisperMode integration

## What Must Be Rewritten In RESense

The Linux port is a backend rewrite, not a small patch.

### Replace Windows Platform Dependencies

These current areas are Windows-specific and need Linux replacements:

- pipe transport
- registry reads/writes
- service checks
- session-specific admin-pipe assumptions

### Replace Fan State Model

Current Windows fan logic depends on:

- live pipe queries for temperatures and RPM
- HKLM for exact active mode and remembered custom state

Linux should instead use:

- EC live reads for RPM, temperatures, and current mode bytes
- EC reads for exact mode where available
- RESense-managed persisted state only if exact remembered custom state is not fully readable

### Replace Keyboard State Model

Current Windows keyboard logic depends on NitroSense XML as the supported source of truth.

Linux has no equivalent. The Linux backend needs:

- direct write support through kernel device or another hardware interface
- live getter support only if a readable interface exists
- otherwise a RESense-managed persisted keyboard state file

## What Needs Reverse Engineering On Linux

### Fan

Needed:

- confirm authoritative EC bytes for CPU fan mode
- confirm authoritative EC bytes for GPU fan mode
- confirm manual speed encoding and valid ranges
- confirm whether mixed auto/manual states are directly readable

### Mode

Needed:

- confirm mode register and exact values on AN515-58
- confirm write behavior is stable across repeated transitions
- confirm behavior on AC and battery if policies differ

### Keyboard

Needed:

- determine whether any real readback path exists
- validate static zone payload semantics
- validate dynamic effect ids
- validate brightness scale
- validate direction encoding
- confirm whether writes fully define state or depend on hidden prior state

### LCD Overdrive

Needed:

- determine whether Linux exposes it through EC, WMI, `acer-wmi`, sysfs, or not at all

## Safe Validation Strategy

Linux should be tested like a hardware reverse-engineering project.

### Principles

- start read-only
- change one thing at a time
- diff before and after state
- prefer reversible tests first
- never fuzz unknown EC addresses
- validate physical behavior, not only software-visible bytes

### Recommended Order

1. Confirm exact machine model via DMI.
2. Bring up EC access without writing.
3. Capture baseline EC snapshots.
4. Toggle one known feature and compare snapshots.
5. Repeat the same action several times to confirm stable bytes.
6. Validate reversible writes:
   - `default -> quiet -> default`
   - `auto -> manual -> auto`
   - keyboard timeout off/on/off
7. Validate small-step manual fan changes.
8. Validate keyboard static writes, then dynamic writes.
9. Validate reboot behavior.

### Validation Requirements Per Feature

For a feature to be considered supported:

- write succeeds at the interface layer
- expected state bytes change as predicted
- physical hardware behavior matches
- repeated writes are idempotent
- rollback path works
- reboot behavior is understood

## Live ISO Expectations

A Linux RESense can work in a live ISO, but only if the runtime environment exposes the required interfaces.

Requirements:

- root access
- EC access path available
- required kernel modules load successfully
- keyboard module/device exists if keyboard RGB is needed

Important caveats:

- out-of-tree modules may fail under Secure Boot
- persisted RESense state may be temporary in a non-persistent live session
- debugfs or similar paths may not be mounted by default

Practical expectation:

- fan and mode are the most realistic live-ISO targets
- keyboard depends on whether the required module can be loaded there

## Recommended Linux Backend Shape

RESense should not try to emulate Windows internals on Linux.

Use separate backends:

- `windows`: existing pipe/registry/XML implementation
- `linux_ec`: fan, mode, timeout, RPM, temperatures
- `linux_keyboard`: keyboard device/module integration

Each feature should expose:

- write path
- read model type
- validation status

## Bottom Line

A proper Linux RESense is realistic for the important controls.

Strong candidates for reliable Linux support:

- fan control
- operation mode control
- RPM and temperature readback

Likely workable:

- keyboard writes

Still open:

- full keyboard live readback
- LCD overdrive

The Linux port should be treated as a first-class backend with its own documented source-of-truth model, not as a compatibility shim over the current Windows design.
