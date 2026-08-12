# Architecture

RESense has one public library facade and one Windows platform implementation. The command-line binary parses arguments, invokes the facade, and renders verified state.

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

`Device` validates the model once, owns the platform context, exposes typed reads and writes, applies operation-mode policy, synchronizes NitroSense persistence, and verifies every mutation. Feature modules do not import Clap or render output.

`Platform` is private to the library. It owns the Acer service connection, Windows session operations, registry access, and atomic file replacement.

The service adapter uses typed requests, exact reads and writes, and centralized reply validation. Feature code receives typed values rather than transport frames.

Registry access uses `winreg`. Vendor-compatible writes go through the Acer service, while registry values provide persisted state where NitroSense uses them.

`lighting_store` resolves the active profile under `C:\ProgramData\OEM\NitroSense`, reads `Main.xml`, and writes through a same-directory temporary file. The temporary file is flushed, parsed, and atomically replaced while unrelated XML nodes are preserved.

Unsafe Windows calls are isolated in `platform/windows.rs` and documented with their safety invariants.

## State Model

Domain types make invalid values difficult to construct:

- `Percent` is `0..=100`.
- `Brightness` is `1..=5`.
- `DynamicSpeed` is `1..=9`.
- `Rgb` is a validated color value.
- Fan state has fixed CPU/GPU fields and distinguishes global auto, max, custom, per-fan auto, and per-fan manual control.
- Keyboard state always has exactly four zones.
- Dynamic effect variants encode whether color and direction exist.
- Operation modes and sound presets are typed enums; unavailable hardware values are optional and invalid protocol values are errors.

Automatic fan changes preserve the remembered manual percentage. They do not invent or overwrite a percentage with `50%`.

## Failure Semantics

Hardware, registry, and XML changes are not one transaction. If a vendor operation succeeds but persistence or verification fails, the command returns an explicit error.

Unavailable values are represented as `null` where that distinction is useful to the caller. Invalid service, registry, XML, or protocol values are errors.
