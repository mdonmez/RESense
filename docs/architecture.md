# Architecture

RESense has one public library facade and one platform implementation. The command-line binary only parses arguments, invokes the facade, and renders verified state.

```text
src/
├── lib.rs
├── main.rs
├── cli.rs
├── app.rs
├── output.rs
├── error.rs
├── device/
│   ├── mod.rs
│   ├── fan.rs
│   ├── keyboard.rs
│   ├── mode.rs
│   ├── display.rs
│   └── sound.rs
├── platform/
│   ├── mod.rs
│   ├── pipe.rs
│   ├── registry.rs
│   ├── lighting_store.rs
│   └── windows.rs
└── bin/
    └── probe.rs
```

## Ownership

`Device` validates the model once, owns the platform context, exposes typed reads and writes, applies quiet-mode and WhisperMode policy, synchronizes NitroSense persistence, and verifies every mutation. Feature modules do not import Clap or render output.

`Platform` is private to the library. It owns the service pipe, current-session admin context, per-operation discovery of admin-session candidates, registry access, Windows session/Sticky Keys operations, and atomic file replacement. No generic backend trait or async runtime is introduced before a second real backend exists.

`pipe` uses `OpenOptions`, `Read::read_exact`, and `Write::write_all`. Requests use borrowed typed arguments; replies are length-checked and decoded to integers before feature code sees them. Raw frames are available only in the `dev-tools` probe binary.

`registry` uses `winreg` for reads. NitroSense-compatible writes still use the vendor service command. Registry errors are not converted to defaults. The model probe is the one deliberate optional-read case because each firmware registry location is an alternative identity source.

`lighting_store` resolves the active profile under `C:\ProgramData\OEM\NitroSense`, reads `Main.xml`, and writes through a same-directory temporary file. The temporary file is flushed, parsed, and atomically replaced. Unknown XML nodes remain in the parsed tree.

The only unsafe code is in `platform/windows.rs`, with explicit safety invariants for session discovery, Sticky Keys readback, and Windows atomic replacement. No service-control-manager startup query or unaligned service-configuration cast remains.

## State Model

Domain types make invalid values difficult to construct:

- `Percent` is `0..=100`.
- `Brightness` is `1..=5`.
- `DynamicSpeed` is `1..=9`.
- `Rgb` is a validated color value.
- Fan state has fixed CPU/GPU fields and distinguishes global auto, max, custom, per-fan auto, and per-fan manual control.
- Keyboard state always has exactly four zones.
- Dynamic effect variants encode whether color and direction exist.
- Operation modes and sound presets are typed enums; unsupported hardware is `Option`, while unknown protocol codes are errors.

Automatic fan changes preserve the remembered manual percentage. They do not invent or overwrite a percentage with `50%`.

## Failure Semantics

The facade does not claim transactional behavior across hardware, registry, and XML. If a vendor operation succeeds but a persistence or verification step fails, the command returns an explicit error. The output layer never turns that error into a successful `null` state.

Known unsupported state is represented as `null` only where the user can act on that distinction: sound when the active output is non-DTS, or display overdrive when the model reports no support. Unknown service, registry, XML, or protocol values are errors.
